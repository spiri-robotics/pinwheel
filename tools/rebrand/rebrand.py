"""rebrand: rename the product in the frontend's user-visible text.

Svelte and TypeScript sources are parsed with tree-sitter, so only text a user
can see is touched: markup text, attribute values, and the contents of string
and template literals. Comments, identifiers, imports, object keys, and strings
the code compares against or uses as values are left alone.

Within that text:

- the product name, as a word in prose, is renamed (`Windmill` -> `Pinwheel`,
  `windmill` -> `pinwheel`, `WINDMILL` -> `PINWHEEL`). Lowercase is only
  renamed inside text that contains a space, since a bare `'windmill'` is
  nearly always a value (an enum member, a mount path, a message tag);
- links to upstream's website (docs, blog, changelog) are pointed at the
  fork's repository, so nothing implies an affiliation with upstream;
- everything that can't be rewritten mechanically is *flagged*: functional
  upstream endpoints (Hub, telemetry, cloud), upstream email addresses,
  upstream products (Hub, Cloud, Enterprise Edition), and the name in a
  position the tool won't touch. Flags are fixed by hand.

Usage:
    rebrand ROOT [--check] [--log FILE] [--update-known]

Without --check, rewrites are applied in place. With --check nothing changes,
and the exit status is 1 if a rewrite is pending or a flag isn't listed in the
known-flags file (`known-flags.txt` next to this script). CI runs --check, so
branding that arrives with an upstream import fails the build until it is
rewritten or acknowledged. --update-known writes the current flags to that file.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from collections import Counter
from dataclasses import dataclass, field
from pathlib import Path

import tree_sitter_svelte
import tree_sitter_typescript
from tree_sitter import Language, Node, Parser

__version__ = "0.1.0"
SVELTE = Language(tree_sitter_svelte.language())
TS = Language(tree_sitter_typescript.language_typescript())

NAMES = {"Windmill": "Pinwheel", "windmill": "pinwheel", "WINDMILL": "PINWHEEL"}
REPO = "https://github.com/spiri-robotics/windmill-OSS"
KNOWN_FLAGS = Path(__file__).with_name("known-flags.txt")

# ---------------------------------------------------------------------------
# Prose rules
# ---------------------------------------------------------------------------
# Upstream products and services. Renaming them would claim we run them, so
# each mention is flagged for a person to reword or remove.
UPSTREAM_PRODUCTS = re.compile(
    r"\bWindmill(?:\s+|-)(?:Hub|Cloud|Labs|Enterprise|EE|Pro|Team|team|Whitelabel|GitHub|Github|managed)\b")

_URL_END = r"""[^\s"'`<>(){}\[\]|\\,;]*"""
UPSTREAM_URL = re.compile(
    r"(?:https?://)?(?:[\w-]+\.)*windmill\.(?:dev|xyz)(?![\w-])" + _URL_END
    + r"|(?:https?://)?(?:github\.com|raw\.githubusercontent\.com|ghcr\.io)/windmill-labs" + _URL_END)
UPSTREAM_EMAIL = re.compile(r"[\w.+-]+@windmill\.dev\b")

# Hosts whose pages are only documentation, so pointing them elsewhere loses
# nothing functional.
DOC_HOSTS = {"www.windmill.dev", "windmill.dev", "docs.windmill.dev"}
DOC_PATHS = re.compile(r"^/(?:docs|blog|changelog|integrations|guides)(?:[/?#]|$)|^/?$")


def name_pattern(name: str) -> re.Pattern:
    # Not part of an identifier, path, host, package name or tag.
    # A hyphenated compound (`Windmill-signed`) is prose; `Windmill-Deploy` is a header.
    before = r"(?<![\w$@/.:\-])"
    after = r"(?![\w@/]|-[^a-z]|-$|[.:]\w)"
    if name.islower():  # lowercase is a value when quoted or bracketed
        before = r"(?<![\w$@/.:\-\[(\"'`<])"
        after = r"(?![\w\-@/\]\"'`>]|[.:]\w)"
    return re.compile(before + re.escape(name) + after)


NAME_PATTERNS = [(name_pattern(old), old, new) for old, new in NAMES.items()]


@dataclass
class Hit:
    start: int  # offsets into the text being scanned
    end: int
    replacement: str | None  # None: flag only
    reason: str


def scan_prose(text: str, followed_by_code: bool = False) -> list[Hit]:
    """Rewrites and flags inside one piece of user-visible text."""
    hits: list[Hit] = []
    taken: list[tuple[int, int]] = []

    def free(a, b):
        return all(b <= s or a >= e for s, e in taken)

    for m in UPSTREAM_EMAIL.finditer(text):
        hits.append(Hit(m.start(), m.end(), None, "upstream email address"))
        taken.append(m.span())
    for m in UPSTREAM_URL.finditer(text):
        if not free(*m.span()):
            continue
        taken.append(m.span())
        new = doc_link_target(m.group())
        if new is None:
            hits.append(Hit(m.start(), m.end(), None, "upstream endpoint; not a doc link"))
        elif followed_by_code and m.end() == len(text):
            hits.append(Hit(m.start(), m.end(), None, "doc link completed by code; rewrite by hand"))
        else:
            hits.append(Hit(m.start(), m.end(), new, "upstream doc link"))
    for m in UPSTREAM_PRODUCTS.finditer(text):
        if free(*m.span()):
            hits.append(Hit(m.start(), m.end(), None, "upstream product or service"))
            taken.append(m.span())
    for pat, old, new in NAME_PATTERNS:
        if old.islower() and not re.search(r"\s", text):
            continue
        for m in pat.finditer(text):
            if free(*m.span()):
                hits.append(Hit(m.start(), m.end(), new, "product name"))
                taken.append(m.span())
    return sorted(hits, key=lambda h: h.start)


def doc_link_target(url: str) -> str | None:
    """Where an upstream doc link should point now, or None if it isn't one."""
    m = re.match(r"(?:(https?)://)?([^/?#]+)(.*)", url)
    scheme, host, path = m.groups()
    if host not in DOC_HOSTS:
        return None
    if scheme is None and host == "windmill.dev":
        return None  # a bare hostname is usually a host check, not a link
    if not DOC_PATHS.match(path):
        return None  # pricing, terms, privacy: remove by hand
    if path.startswith("/changelog"):
        return f"{REPO}/releases"
    return REPO


def mentions_brand(text: str) -> bool:
    return bool(re.search(r"(?<![\w$])(?:Windmill|WINDMILL)(?![\w])", text)
                or UPSTREAM_URL.search(text) or UPSTREAM_EMAIL.search(text))


# ---------------------------------------------------------------------------
# Edits and logging
# ---------------------------------------------------------------------------


@dataclass
class Edit:
    start: int
    end: int
    replacement: bytes


@dataclass
class Report:
    records: list[dict] = field(default_factory=list)
    flags: list[dict] = field(default_factory=list)

    def add(self, rec: dict) -> None:
        (self.flags if rec["action"] == "flag" else self.records).append(rec)


def line_of(src: bytes, offset: int) -> int:
    return src.count(b"\n", 0, offset) + 1


class FileScanner:
    def __init__(self, rel: str, src: bytes, report: Report):
        self.rel = rel
        self.src = src
        self.report = report
        self.edits: list[Edit] = []

    def text(self, start: int, end: int, kind: str, followed_by_code: bool = False) -> None:
        """Scan src[start:end], a piece of user-visible text."""
        raw = self.src[start:end]
        if not mentions_brand(raw.decode(errors="replace")) and b"windmill" not in raw:
            return
        text = raw.decode()
        for h in scan_prose(text, followed_by_code):
            # Offsets in `text` are characters; edits need bytes.
            s = start + len(text[:h.start].encode())
            e = start + len(text[:h.end].encode())
            rec = {"file": self.rel, "line": line_of(self.src, s), "kind": kind,
                   "match": text[h.start:h.end], "reason": h.reason}
            if h.replacement is None:
                self.report.add({"action": "flag", **rec})
                continue
            self.edits.append(Edit(s, e, h.replacement.encode()))
            self.report.add({"action": "rewrite", **rec, "result": h.replacement})

    def value(self, node: Node, why: str) -> None:
        """A string the tool won't change because the code depends on it."""
        text = node.text.decode(errors="replace")
        for m in re.finditer(r"(?<![\w$])(?:Windmill|WINDMILL)(?![\w])", text):
            self.report.add({"action": "flag", "file": self.rel, "line": line_of(self.src, node.start_byte),
                             "kind": "value", "match": m.group(), "reason": f"name in {why}; not rewritten"})
        for m in UPSTREAM_URL.finditer(text):
            self.report.add({"action": "flag", "file": self.rel, "line": line_of(self.src, node.start_byte),
                             "kind": "value", "match": m.group(), "reason": f"upstream URL in {why}; not rewritten"})


def is_test(rel: str) -> bool:
    return bool(re.search(r"\.(test|spec)\.[jt]s$", rel)) or rel.startswith("e2e/") or "/e2e/" in rel


# ---------------------------------------------------------------------------
# TypeScript
# ---------------------------------------------------------------------------

COMPARISONS = {"===", "!==", "==", "!=", "in", "instanceof"}
# Methods whose string arguments are keys, patterns or selectors.
VALUE_METHODS = {
    "includes", "indexOf", "lastIndexOf", "startsWith", "endsWith", "match", "matchAll",
    "replace", "replaceAll", "split", "search", "test", "getItem", "setItem", "removeItem",
    "querySelector", "querySelectorAll", "getElementById", "has", "get", "set", "delete",
    "getAttribute", "setAttribute", "addEventListener", "removeEventListener", "postMessage",
    "require", "import",
}
# Object keys whose string values are identifiers rather than display text.
VALUE_KEYS = {"value", "id", "key", "type", "kind", "mode", "path", "provider", "clientID", "slug",
              "language", "lang", "event", "channel", "storageKey", "resource_type"}


def value_context(string: Node) -> str | None:
    """Why a string literal is a value the code depends on, or None if it's text."""
    node, parent = string, string.parent
    while parent is not None and parent.type in ("parenthesized_expression", "as_expression",
                                                 "satisfies_expression", "non_null_expression"):
        node, parent = parent, parent.parent
    if parent is None:
        return None
    t = parent.type
    if t in ("import_statement", "export_statement", "import_require_clause", "literal_type",
             "enum_assignment", "property_signature", "index_signature"):
        return t
    if t == "pair" and parent.child_by_field_name("key") == node:
        return "object key"
    if t == "pair":
        key = parent.child_by_field_name("key")
        if key is not None and key.text.decode().strip("'\"") in VALUE_KEYS:
            return f"`{key.text.decode()}` property"
    if t == "subscript_expression" and parent.child_by_field_name("index") == node:
        return "index"
    if t == "binary_expression":
        op = parent.child_by_field_name("operator")
        if op is not None and op.type in COMPARISONS:
            return "comparison"
    if t == "switch_case":
        return "switch case"
    if t == "arguments":
        call = parent.parent
        fn = call.child_by_field_name("function") if call is not None else None
        if fn is not None:
            name = fn.child_by_field_name("property") if fn.type == "member_expression" else fn
            if name is not None and name.text.decode() in VALUE_METHODS:
                return f"{name.text.decode()}() argument"
    if t in ("call_expression",) and node.type == "template_string":
        return "tagged template"
    return None


def scan_ts(fs: FileScanner, src: bytes, base: int, as_expression: bool = False) -> None:
    """Scan TypeScript source `src`, which sits at byte offset `base` in the file."""
    if as_expression:
        tree = Parser(TS).parse(b"(" + src + b")")
        base -= 1
    else:
        tree = Parser(TS).parse(src)

    def visit(node: Node):
        if node.type in ("string", "template_string"):
            why = value_context(node)
            if why is not None:
                fs.value(node, why)
                return
            frags = [c for c in node.children if c.type == "string_fragment"]
            for c in frags:
                nxt = c.next_sibling
                fs.text(base + c.start_byte, base + c.end_byte, node.type,
                        followed_by_code=nxt is not None and nxt.type == "template_substitution")
            # Substitutions hold code, possibly with more strings.
            for c in node.children:
                if c.type == "template_substitution":
                    visit(c)
            return
        if node.type in ("comment", "regex"):
            return
        for c in node.children:
            visit(c)

    visit(tree.root_node)


# ---------------------------------------------------------------------------
# Svelte (and plain HTML)
# ---------------------------------------------------------------------------

# Attributes that hold identifiers, classes, or code; their values keep the
# name but are still scanned for links.
CODE_ATTRS = {"id", "class", "style", "name", "for", "type", "value", "lang", "rel", "slot", "key",
              "src", "target", "role", "method", "action", "context", "this", "xmlns", "fill",
              "viewBox", "d"}


def code_attr(name: str) -> bool:
    return name in CODE_ATTRS or re.match(r"(data|bind|on|use|class|style|transition|in|out|animate|let):", name) \
        is not None or name.startswith("data-")


def scan_svelte(fs: FileScanner) -> None:
    tree = Parser(SVELTE).parse(fs.src)

    def visit(node: Node):
        t = node.type
        if t == "script_element":
            raw = next((c for c in node.children if c.type == "raw_text"), None)
            if raw is not None and not is_json_script(node):
                scan_ts(fs, raw.text, raw.start_byte)
            return
        if t in ("style_element", "comment"):
            return
        if t == "svelte_raw_text":
            scan_ts(fs, node.text, node.start_byte, as_expression=True)
            return
        if t == "text":
            fs.text(node.start_byte, node.end_byte, "markup")
            return
        if t == "attribute":
            name_node = node.child_by_field_name("name") or next(
                (c for c in node.children if c.type == "attribute_name"), None)
            name = name_node.text.decode() if name_node is not None else ""
            for c in node.children:
                if c.type in ("quoted_attribute_value", "attribute_value"):
                    for v in ([c] if c.type == "attribute_value" else c.children):
                        if v.type != "attribute_value":
                            continue
                        if code_attr(name):
                            links_only(fs, v.start_byte, v.end_byte)
                        else:
                            fs.text(v.start_byte, v.end_byte, f"attribute {name}")
                else:
                    visit(c)
            return
        for c in node.children:
            visit(c)

    visit(tree.root_node)


def is_json_script(node: Node) -> bool:
    return b'type="application/json"' in node.text[:200] or b"application/ld+json" in node.text[:200]


def links_only(fs: FileScanner, start: int, end: int) -> None:
    """Scan a code attribute's value for upstream links only."""
    text = fs.src[start:end].decode()
    for m in UPSTREAM_URL.finditer(text):
        fs.text(start + len(text[:m.start()].encode()), start + len(text[:m.end()].encode()), "link attribute")


# ---------------------------------------------------------------------------
# Plain text (prompt YAML)
# ---------------------------------------------------------------------------


def scan_plain(fs: FileScanner) -> None:
    pos = 0
    for line in fs.src.splitlines(keepends=True):
        fs.text(pos, pos + len(line), "text")
        pos += len(line)


# ---------------------------------------------------------------------------
# Driver
# ---------------------------------------------------------------------------

SKIP_DIRS = {"node_modules", ".svelte-kit", "build", "dist", "gen"}


def relevant(rel: str) -> bool:
    parts = rel.split("/")
    if SKIP_DIRS.intersection(parts[:-1]):
        return False
    if rel.endswith(".gen.ts") or rel.endswith(".d.ts"):
        return False  # generated, or rewritten upstream of us
    if is_test(rel):
        return False  # fixtures, not text anyone sees; some assert on prompts we don't rewrite
    if parts[0] not in ("src", "static"):
        return False
    return rel.endswith((".svelte", ".ts", ".js", ".html", ".yaml"))


def apply_edits(src: bytes, edits: list[Edit]) -> bytes:
    edits = sorted(edits, key=lambda e: e.start)
    for a, b in zip(edits, edits[1:]):
        if b.start < a.end:
            raise RuntimeError(f"overlapping edits at bytes {a.start}-{a.end} / {b.start}-{b.end}")
    out = bytearray(src)
    for e in reversed(edits):
        out[e.start:e.end] = e.replacement
    return bytes(out)


def rewrite_source(rel: str, src: bytes, report: Report) -> bytes:
    fs = FileScanner(rel, src, report)
    if rel.endswith((".svelte", ".html")):
        scan_svelte(fs)
    elif rel.endswith(".yaml"):
        scan_plain(fs)
    else:
        scan_ts(fs, src, 0)
    return apply_edits(src, fs.edits)


def source_files(root: Path) -> list[str]:
    """Tracked files under root, so build output (e.g. an unpacked ui_builder) is skipped."""
    try:
        out = subprocess.run(["git", "-C", str(root), "ls-files", "-z"], check=True,
                             capture_output=True).stdout
        return sorted(p.decode() for p in out.split(b"\0") if p)
    except (subprocess.CalledProcessError, FileNotFoundError):
        return sorted(p.relative_to(root).as_posix() for p in root.rglob("*") if p.is_file())


def run(root: Path, dry_run: bool) -> Report:
    root = root.resolve()
    report = Report()
    for rel in source_files(root):
        p = root / rel
        if not p.is_file() or p.is_symlink() or not relevant(rel):
            continue
        src = p.read_bytes()
        if b"indmill" not in src and b"INDMILL" not in src:
            continue
        out = rewrite_source(rel, src, report)
        if out != src and not dry_run:
            p.write_bytes(out)
    return report


def flag_key(rec: dict) -> str:
    """A flag's identity for the known list: stable across line moves."""
    return f"{rec['file']}\t{rec['match']}\t{rec['reason']}"


def read_known(path: Path) -> Counter:
    if not path.exists():
        return Counter()
    lines = [l for l in path.read_text().splitlines() if l and not l.startswith("#")]
    return Counter(lines)


def write_known(path: Path, report: Report) -> None:
    keys = sorted(flag_key(f) for f in report.flags)
    path.write_text("# Flags acknowledged by tools/rebrand: file, match, reason (one line per occurrence).\n"
                    "# Fix them by hand and delete their lines. Regenerate with --update-known.\n"
                    + "".join(k + "\n" for k in keys))


def tool_version() -> str:
    digest = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()[:12]
    return f"{__version__}+{digest}"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("root", type=Path, help="the frontend directory")
    ap.add_argument("--check", action="store_true", help="change nothing; fail on pending rewrites or new flags")
    ap.add_argument("--log", type=Path, default=None, help="write every rewrite and flag here (JSON lines)")
    ap.add_argument("--known", type=Path, default=KNOWN_FLAGS, help="acknowledged flags")
    ap.add_argument("--update-known", action="store_true", help="record the current flags as acknowledged")
    args = ap.parse_args(argv)

    report = run(args.root, dry_run=args.check)
    if args.log:
        with args.log.open("w") as fh:
            fh.write(json.dumps({"tool": tool_version(), "check": args.check}) + "\n")
            for rec in report.records + report.flags:
                fh.write(json.dumps(rec) + "\n")
    if args.update_known:
        write_known(args.known, report)

    new_flags = Counter(flag_key(f) for f in report.flags) - read_known(args.known)
    gone = read_known(args.known) - Counter(flag_key(f) for f in report.flags)
    verb = "pending" if args.check else "applied"
    print(f"rebrand {tool_version()}: {len(report.records)} rewrite(s) {verb}, "
          f"{len(report.flags)} flag(s), {sum(new_flags.values())} not in {args.known.name}")
    if args.check:
        for rec in report.records[:50]:
            print(f"  rewrite {rec['file']}:{rec['line']}: {rec['match']!r} -> {rec['result']!r}")
    by_key = {flag_key(f): f for f in report.flags}
    for key in sorted(new_flags):
        f = by_key[key]
        print(f"  flag {f['file']}:{f['line']}: {f['match']!r} ({f['reason']})")
    if gone and not args.update_known:
        print(f"  {sum(gone.values())} acknowledged flag(s) no longer occur; run --update-known to drop them")
    if args.check and (report.records or new_flags):
        print("::error::Upstream branding found. Run `uv run --project tools/rebrand rebrand frontend`, "
              "fix or acknowledge the flags, and commit.")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
