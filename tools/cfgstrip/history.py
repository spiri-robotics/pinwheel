"""cfgstrip-history: rewrite a git history so every commit is stripped.

Each upstream commit becomes exactly one rewritten commit: same author,
committer, dates and message, with its tree passed through cfgstrip and a
`Filtered-by: cfgstrip <version>` trailer appended. Merge structure is kept.

Runs are incremental without any state file. Rewriting keeps each commit's
author, committer, dates and message, so the existing rewritten branch says by
itself which upstream commits it already covers: a later run matches them up,
and rewrites only the upstream commits that are new, stacked on their matched
parents, with whatever version of cfgstrip is current. To redo history from
some point (say a later tool version handles a commit better), reset the
rewritten branch to just before that commit and run again.

Every commit gets the trailer, including ones the filter doesn't otherwise
change, so the rewritten history shares no commits with upstream. Git then
refuses an accidental raw merge of upstream ("unrelated histories").

`--freeze PATH` keeps a top-level path as the first parent's rewritten commit
has it, so upstream's changes there never arrive (the fork uses this for
`.github`: it owns its own CI, and GitHub won't let a workflow token push
commits that touch workflow files).

Usage:
    cfgstrip-history --repo UPSTREAM.git --ref v1.2.3 --branch upstream-stripped \\
        [--drop AGENTS.md --drop .claude ...] [--freeze .github]

UPSTREAM.git must also contain the existing rewritten branch, if any (fetch it
from the fork first). New objects are written into UPSTREAM.git and
`refs/heads/<branch>` there is pointed at the rewritten tip. Publish by pushing that branch; a push only sends
objects reachable from it, so none of the unstripped history goes along.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import time
from functools import lru_cache
from pathlib import Path

import pygit2

import cfgstrip

TREE = pygit2.GIT_OBJECT_TREE
TRAILER_LINE = re.compile(rb"^[A-Za-z0-9][A-Za-z0-9-]*: .+$")


def add_trailer(message: bytes, trailer: bytes) -> bytes:
    body = message.rstrip(b"\n")
    last_para = body.rsplit(b"\n\n", 1)[-1]
    lines = last_para.split(b"\n")
    in_trailer_block = b"\n\n" in body and all(TRAILER_LINE.match(l) for l in lines if l)
    sep = b"\n" if in_trailer_block else b"\n\n"
    return body + sep + trailer + b"\n"


class Rewriter:
    def __init__(self, repo: pygit2.Repository, off: set[str], drop: set[str], version: str,
                 freeze: set[str] = frozenset()):
        self.repo = repo
        self.off = off
        self.drop = drop
        self.freeze = freeze
        self.version = version
        self.tree_memo: dict[pygit2.Oid, pygit2.Oid] = {}
        self.strip_memo: dict[tuple, dict[str, pygit2.Oid | None]] = {}
        self.file_cache: dict = {}  # per-blob work shared across trees (see cfgstrip.strip_tree)
        self.stats = {"commits": 0, "trees_stripped": 0, "files_changed": 0, "files_deleted": 0,
                      "flags": 0, "residual": 0}
        self.residual: list[dict] = []
        self.has_rel = lru_cache(maxsize=200_000)(self._has_rel)
        self.rel_files = lru_cache(maxsize=4096)(self._rel_files)

    # -- tree inspection ------------------------------------------------------
    def _has_rel(self, oid: pygit2.Oid) -> bool:
        for e in self.repo[oid]:
            if e.type == TREE:
                if e.name not in cfgstrip.SKIP_DIRS and self.has_rel(e.id):
                    return True
            elif cfgstrip.relevant(e.name):
                return True
        return False

    def _rel_files(self, oid: pygit2.Oid) -> tuple[tuple[str, pygit2.Oid], ...]:
        """(relative path, blob id) of every relevant file under a tree."""
        out = []
        for e in self.repo[oid]:
            if e.type == TREE:
                if e.name in cfgstrip.SKIP_DIRS or not self.has_rel(e.id):
                    continue
                out.extend((f"{e.name}/{p}", b) for p, b in self.rel_files(e.id))
            elif cfgstrip.relevant(e.name) and e.filemode != pygit2.GIT_FILEMODE_LINK:
                out.append((e.name, e.id))
        return tuple(out)

    # -- stripping ------------------------------------------------------------
    def strip(self, root: pygit2.Tree) -> dict[str, pygit2.Oid | None]:
        # Only root entries holding .rs/Cargo.toml files can change the result,
        # so commits that touch nothing else (frontend, docs) reuse it.
        key = tuple(sorted((e.name, e.id) for e in root
                           if e.type == TREE and e.name not in self.drop and self.has_rel(e.id)))
        key += tuple(sorted((e.name, e.id) for e in root if e.type != TREE and cfgstrip.relevant(e.name)))
        hit = self.strip_memo.get(key)
        if hit is not None:
            return hit
        files: dict[str, pygit2.Oid] = {}
        for e in root:
            if e.name in self.drop:
                continue
            if e.type == TREE:
                if e.name not in cfgstrip.SKIP_DIRS and self.has_rel(e.id):
                    files.update((f"{e.name}/{p}", b) for p, b in self.rel_files(e.id))
            elif cfgstrip.relevant(e.name):
                files[e.name] = e.id

        report = cfgstrip.Report()
        changes = cfgstrip.strip_tree(set(files), lambda p: self.repo[files[p]].data, self.off, report,
                                      content_id=lambda p: files[p], cache=self.file_cache)
        self.stats["trees_stripped"] += 1
        self.stats["flags"] += len(report.flags)
        result: dict[str, pygit2.Oid | None] = {}
        for path, data in changes.items():
            result[path] = None if data is None else self.repo.create_blob(data)

        # Safety net: no Rust file in the result may still gate on an off feature.
        # Comment lines don't count: commented-out code isn't compiled under any flag.
        gate = re.compile(rb'^(?!\s*//).*feature\s*=\s*"(' +
                          b"|".join(re.escape(f.encode()) for f in sorted(self.off)) + rb')"', re.M)
        for path, oid in files.items():
            if not path.endswith(".rs"):
                continue
            new = result.get(path, oid)
            if new is None:
                continue
            if gate.search(self.repo[new].data):
                self.residual.append({"path": path})
        self.strip_memo[key] = result
        return result

    # -- tree rebuilding ------------------------------------------------------
    def rebuild(self, tree: pygit2.Tree, changes: dict[str, pygit2.Oid | None],
                drop: set[str] = frozenset()) -> pygit2.Oid | None:
        """Apply {path: blob|None} to a tree; returns None if the tree ends up empty."""
        here: dict[str, pygit2.Oid | None] = {}
        below: dict[str, dict[str, pygit2.Oid | None]] = {}
        for path, oid in changes.items():
            head, _, rest = path.partition("/")
            if rest:
                below.setdefault(head, {})[rest] = oid
            else:
                here[head] = oid
        tb = self.repo.TreeBuilder(tree)
        for name in drop:
            if tree.__contains__(name):
                tb.remove(name)
        for name, oid in here.items():
            if oid is None:
                if name in tree:
                    tb.remove(name)
            else:
                tb.insert(name, oid, tree[name].filemode if name in tree else pygit2.GIT_FILEMODE_BLOB)
        for name, sub in below.items():
            if name not in tree or tree[name].type != TREE:
                continue
            new_sub = self.rebuild(self.repo[tree[name].id], sub)
            if new_sub is None:
                tb.remove(name)
            else:
                tb.insert(name, new_sub, pygit2.GIT_FILEMODE_TREE)
        if len(tb) == 0:
            return None
        return tb.write()

    def rewrite_tree(self, tree_id: pygit2.Oid) -> pygit2.Oid:
        hit = self.tree_memo.get(tree_id)
        if hit is not None:
            return hit
        root = self.repo[tree_id]
        changes = self.strip(root)
        self.stats["files_changed"] += sum(1 for v in changes.values() if v is not None)
        self.stats["files_deleted"] += sum(1 for v in changes.values() if v is None)
        new = self.rebuild(root, changes, {d for d in self.drop if d in root})
        if new is None:
            new = self.repo.TreeBuilder().write()
        self.tree_memo[tree_id] = new
        return new

    def freeze_tree(self, tree_id: pygit2.Oid, parent: pygit2.Oid | None) -> pygit2.Oid:
        """Give frozen top-level entries the first parent's version (none for a root commit)."""
        tree = self.repo[tree_id]
        before = self.repo[parent].tree if parent is not None else None
        tb = self.repo.TreeBuilder(tree)
        for name in self.freeze:
            if name in tree:
                tb.remove(name)
            if before is not None and name in before:
                tb.insert(name, before[name].id, before[name].filemode)
        return tb.write()

    # -- commits --------------------------------------------------------------
    def rewrite_commit(self, sha: str, parents: list[str], mapping: dict[str, str]) -> str:
        c = self.repo[sha]
        tree = self.rewrite_tree(c.tree_id)
        msg = add_trailer(c.raw_message, f"Filtered-by: cfgstrip {self.version}".encode())
        parent_ids = [pygit2.Oid(hex=mapping[p]) for p in parents]
        if self.freeze:
            tree = self.freeze_tree(tree, parent_ids[0] if parent_ids else None)
        if c.message_encoding:
            new = self.repo.create_commit(None, c.author, c.committer, msg, tree, parent_ids,
                                          c.message_encoding)
        else:
            new = self.repo.create_commit(None, c.author, c.committer, msg, tree, parent_ids)
        self.stats["commits"] += 1
        return str(new)


TRAILER_PREFIX = b"\nFiltered-by: cfgstrip "


def match_key(c: pygit2.Commit, rewritten: bool) -> tuple:
    """What a commit and its rewrite have in common: author, dates, message."""
    msg = c.raw_message.rstrip(b"\n")
    if rewritten:
        cut = msg.rfind(TRAILER_PREFIX)
        if cut == -1:
            return ()
        msg = msg[:cut].rstrip(b"\n")
    a, m = c.author, c.committer
    return (a.raw_name, a.raw_email, a.time, a.offset, m.raw_name, m.raw_email, m.time, m.offset, msg)


def existing_mapping(repo: pygit2.Repository, branch: str, upstream: list[str]) -> dict[str, str]:
    """Upstream commit -> rewritten commit, recovered from an existing rewritten branch."""
    try:
        tip = repo.references[f"refs/heads/{branch}"].target
    except KeyError:
        return {}
    index: dict[tuple, str] = {}
    ambiguous: set[tuple] = set()
    for c in repo.walk(tip, pygit2.GIT_SORT_NONE):
        k = match_key(c, rewritten=True)
        if not k:
            continue
        if k in index:
            ambiguous.add(k)
        index[k] = str(c.id)
    mapping = {}
    for sha in upstream:
        k = match_key(repo[sha], rewritten=False)
        if k in ambiguous:
            raise SystemExit(f"cannot tell which rewritten commit {sha} maps to (duplicate metadata)")
        if k in index:
            mapping[sha] = index[k]
    return mapping


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--repo", required=True, type=Path, help="repository holding the upstream history")
    ap.add_argument("--ref", required=True, help="upstream commit or ref to import up to")
    ap.add_argument("--branch", default="upstream-stripped", help="rewritten branch to extend")
    ap.add_argument("--off", action="append", default=None)
    ap.add_argument("--drop", action="append", default=[], help="top-level path to remove from every commit")
    ap.add_argument("--freeze", action="append", default=[],
                    help="top-level path that new commits keep from their first parent, ignoring upstream's changes")
    args = ap.parse_args(argv)

    off = set(args.off or cfgstrip.DEFAULT_OFF)
    drop = set(args.drop)
    freeze = set(args.freeze)
    repo = pygit2.Repository(str(args.repo))
    version = cfgstrip.tool_version()

    tip = subprocess.run(["git", "-C", str(args.repo), "rev-parse", "--verify", f"{args.ref}^{{commit}}"],
                         check=True, capture_output=True, text=True).stdout.strip()
    revs = [line.split() for line in subprocess.run(
        ["git", "-C", str(args.repo), "rev-list", "--topo-order", "--reverse", "--parents", tip],
        check=True, capture_output=True, text=True).stdout.split("\n") if line]
    mapping = existing_mapping(repo, args.branch, [r[0] for r in revs])
    todo = [r for r in revs if r[0] not in mapping]
    print(f"cfgstrip {version}: {len(todo)} new commit(s) to rewrite, {len(mapping)} already imported",
          flush=True)
    if not todo:
        return 0

    rw = Rewriter(repo, off, drop, version, freeze)
    t0 = time.time()
    for i, (sha, *parents) in enumerate(todo, 1):
        mapping[sha] = rw.rewrite_commit(sha, parents, mapping)
        if i % 500 == 0:
            print(f"  {i}/{len(todo)}  {i / (time.time() - t0):.1f} commits/s", flush=True)

    new_tip = mapping[tip]
    repo.references.create(f"refs/heads/{args.branch}", pygit2.Oid(hex=new_tip), force=True)
    print(f"done in {time.time() - t0:.0f}s: {json.dumps(rw.stats)}")
    print(f"refs/heads/{args.branch} -> {new_tip}")
    if rw.residual:
        paths = sorted({r["path"] for r in rw.residual})
        print(f"WARNING: {len(paths)} path(s) still gate on an off feature in the new commits:")
        for p in paths[:30]:
            print(f"  {p}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
