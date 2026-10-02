# Verification of the stripped tree

Upstream commit: `fee401f01eebe411675a19eeff07b6f5435660b1` (2026-10-01).
Tool: `tools/cfgstrip`, with `enterprise`, `private` and `enterprise_saml` off.
Log: `tools/cfgstrip/logs/strip-fee401f.jsonl`.

## What was removed

| Action | Count |
|---|---|
| Blocks deleted (items, statements, fields, arms, macro tokens) | 776 |
| Files deleted (always-false `#![cfg]`, or only reached by a deleted `mod`) | 24 |
| `if cfg!(...)` branches folded | 6 |
| Other `cfg!(...)` replaced with a literal | 5 |
| Attributes dropped (now always true) | 489 |
| Attributes rewritten (simplified) | 70 |
| Cargo features removed / references dropped | 83 / 7 |
| **Lines removed** | **35,747** |

Most of the volume is test suites for the removed features, and blocks gated on
`private` inside otherwise public files. No `cfg` predicate mentioning
`enterprise`, `private` or `enterprise_saml` remains in any `.rs` file.

## Checks

Run inside `nix-shell shell.nix`, with a shared target directory:

| Check | Result |
|---|---|
| `cargo check --features oss`, stripped tree | passes |
| `cargo check --features oss`, unmodified upstream (baseline) | passes |
| `cargo check --features oss --tests`, stripped tree | passes |
| Warnings, stripped vs upstream | +16 in the stripped tree, all unused variables, imports or helpers left over by removed code; none in upstream that the stripped tree lacks |
| `tools/cfgstrip` test suite | 27 passed |

## Not yet checked

- Running the backend test suite (needs Postgres).
- A runtime smoke test: login, script run, flow run.
- Feature combinations beyond `oss` (e.g. `kafka`, `nats`, `mssql`, `tantivy`).
  Some of these were enterprise-only upstream and may not build without the
  removed code.

## History rewrite

Every upstream commit up to `fee401f` was rewritten with
`tools/cfgstrip/history.py` (`cfgstrip 0.2.0+167e2b2a3749`), dropping
`AGENTS.md`, `CLAUDE.md`, `.claude/` and `.agents/` from every commit.

| Check | Result |
|---|---|
| Commits rewritten | 14,995 (1,543 merges), 1:1 with upstream; authors, dates and messages kept |
| Shared commits with upstream | none (every commit carries a `Filtered-by` trailer) |
| Rewritten tip vs. the verified stripped snapshot | identical file for file |
| Incremental import (to `fee401f~300`, then to `fee401f`) vs. one full run | same tip hash: the rewrite is deterministic |
| Recovering which upstream commits are imported, by matching author/dates/message | 14,995 of 14,995 matched, all to the right rewritten commit |
| Non-comment `enterprise`/`private`/`enterprise_saml` gates in any blob of any commit | none, except one historical version of `backend/windmill-api/src/job_helpers.rs` (Jan 2024) where a parse error left an always-true `#[cfg(not(feature = "enterprise"))]` on open source code |

Commented-out code mentioning those features remains in some historical
commits. It isn't compiled under any flag, so we don't treat it as code under
the compile flag.
