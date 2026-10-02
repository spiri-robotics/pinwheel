# AGENTS.md

This is Spiri's AGPLv3 fork of Windmill. Read `FORK.md` before doing anything
else: it has the background, the ground rules and the work list.

## The one rule that matters most

Don't look at Windmill's upstream source, history or published binaries for
anything this fork removed. That means:

- no fetching from `github.com/windmill-labs/*` or `raw.githubusercontent.com`
  (the project settings deny these),
- no cloning, fetching or adding upstream as a git remote,
- no pulling or unpacking `ghcr.io/windmill-labs/*` images.

Code was removed from this fork as a precaution. We consider it AGPLv3, but we
don't want anything in the fork to be derived from it. If you need behaviour
that was removed, design it from the surrounding code, the public
documentation and the OpenAPI spec (`backend/windmill-api/openapi.yaml`).
`tools/cfgstrip/logs/` lists where code was removed, without its contents.

## Upstream changes

Never merge upstream's commits directly (`git merge upstream/...`, GitHub's
"Sync fork"). Upstream work arrives through `tools/import-upstream` as
rewritten commits on the `upstream-stripped` branch, which is then merged
normally. CI fails if gated code reappears.

## Building

`nix-shell shell.nix`, then `cd backend && cargo check --features oss`.
