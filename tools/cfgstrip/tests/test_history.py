import json
import subprocess
import textwrap
from pathlib import Path

import history

CARGO = textwrap.dedent('''
    [package]
    name = "app"
    version = "0.1.0"

    [features]
    enterprise = []
    parquet = []
''')


def git(repo: Path, *args: str) -> str:
    env = {"GIT_AUTHOR_NAME": "Up Stream", "GIT_AUTHOR_EMAIL": "up@example.com",
           "GIT_COMMITTER_NAME": "Up Stream", "GIT_COMMITTER_EMAIL": "up@example.com",
           "GIT_AUTHOR_DATE": "2024-01-01T00:00:00Z", "GIT_COMMITTER_DATE": "2024-01-01T00:00:00Z",
           "PATH": "/run/current-system/sw/bin:/usr/bin:/bin", "HOME": str(repo)}
    return subprocess.run(["git", "-C", str(repo), *args], check=True, capture_output=True,
                          text=True, env=env).stdout.strip()


def commit(repo: Path, files: dict[str, str | None], msg: str) -> str:
    for path, text in files.items():
        p = repo / path
        if text is None:
            p.unlink()
        else:
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(text)
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", msg)
    return git(repo, "rev-parse", "HEAD")


def make_upstream(tmp: Path) -> Path:
    up = tmp / "up"
    up.mkdir()
    git(up, "init", "-q", "-b", "main")
    commit(up, {"README.md": "hello\n", "Cargo.toml": CARGO,
                "src/lib.rs": "pub fn open() {}\n"}, "initial")
    commit(up, {"src/lib.rs": 'pub fn open() {}\n#[cfg(feature = "enterprise")]\npub fn secret_one() {}\n',
                "AGENTS.md": "work in the private repo\n"}, "feat(ee): add secret one")
    git(up, "checkout", "-q", "-b", "side")
    commit(up, {"docs.md": "side branch docs\n"}, "docs on a side branch")
    git(up, "checkout", "-q", "main")
    commit(up, {"src/extra.rs": "pub fn extra() {}\n"}, "add extra\n\nSigned-off-by: Up Stream <up@example.com>")
    git(up, "merge", "-q", "--no-ff", "side", "-m", "merge side")
    commit(up, {"src/lib.rs": 'pub fn open() {}\n#[cfg(feature = "enterprise")]\npub fn secret_two() {}\n'},
           "feat(ee): secret two")
    return up


def run(up: Path, ref: str, branch: str = "upstream-stripped") -> str:
    rc = history.main(["--repo", str(up), "--ref", ref, "--branch", branch, "--drop", "AGENTS.md"])
    assert rc == 0
    return git(up, "rev-parse", branch)


def test_history_rewrite(tmp_path: Path):
    up = make_upstream(tmp_path)
    tip = run(up, "main")

    # every rewritten commit is free of the gated code and the dropped file
    for sha in git(up, "rev-list", tip).split():
        files = git(up, "ls-tree", "-r", "--name-only", sha).split()
        assert "AGENTS.md" not in files
        assert "secret" not in git(up, "show", f"{sha}:src/lib.rs")
        msg = git(up, "log", "-1", "--format=%B", sha)
        assert "Filtered-by: cfgstrip " in msg

    # structure, authorship and messages preserved; no shared commits with upstream
    assert git(up, "rev-list", "--count", tip) == git(up, "rev-list", "--count", "main")
    assert git(up, "rev-list", "--count", "--merges", tip) == "1"
    assert git(up, "log", "-1", "--format=%an %ad", "--date=iso-strict", tip) == \
        git(up, "log", "-1", "--format=%an %ad", "--date=iso-strict", "main")
    assert not set(git(up, "rev-list", tip).split()) & set(git(up, "rev-list", "main").split())
    trailer_msg = git(up, "log", "-1", "--format=%B", f"{tip}~1^1")  # the "add extra" commit
    assert "Signed-off-by: Up Stream <up@example.com>\nFiltered-by: cfgstrip" in trailer_msg

    # a second run finds everything already imported and changes nothing
    assert run(up, "main") == tip


def test_incremental_equals_full(tmp_path: Path):
    up = make_upstream(tmp_path)
    full = run(up, "main", branch="full")
    run(up, "main~1", branch="inc")
    inc = run(up, "main", branch="inc")
    assert inc == full


def test_redo_from_a_point(tmp_path: Path):
    up = make_upstream(tmp_path)
    tip = run(up, "main")
    # reset the rewritten branch to before the last commit; only that one is redone
    git(up, "update-ref", "refs/heads/upstream-stripped", "upstream-stripped~1")
    assert run(up, "main") == tip
