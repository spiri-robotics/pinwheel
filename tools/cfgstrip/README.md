# cfgstrip

Removes Cargo-feature-gated code from a Rust source tree. Every `#[cfg(...)]`,
`#[cfg_attr(...)]` and `cfg!(...)` predicate is partially evaluated with a
chosen set of features forced **off**, and every other feature left unknown.

| Predicate after evaluation | What happens |
|---|---|
| always false | the attributed item, statement, field, arm or `if`-branch is deleted, along with comments directly above it |
| always true | the attribute is dropped and the code is kept |
| still depends on other features | the attribute is rewritten in simplified form (`all(not(feature = "private"), feature = "oauth2")` becomes `feature = "oauth2"`) |

It also:

- folds `if cfg!(...)` conditions and keeps only the live branch, and replaces other `cfg!(...)` uses with `true` or `false`;
- handles `#[cfg]` inside macro bodies at the token level (`lazy_static!` statics, `tokio::select!` arms);
- deletes whole files whose `#![cfg(...)]` is always false, and module files that only an always-false `mod` declaration pulled in;
- removes the off features from every `Cargo.toml`, along with every reference to them (`crate/feature` entries and dependency `features = [...]` lists).

## Why it exists

We use it on Windmill, which ships some code inside AGPLv3-licensed files behind
`enterprise`/`private` compile flags. We consider all of that code AGPLv3, as
the license headers on those files say. We strip it anyway, purely as a
precaution, so that a fork gives no pretext for a nuisance dispute. Upstream's
own `oss` feature bundle already compiles without those flags, so stripping
changes no behaviour of the open source build. It only removes code that build
never compiled.

## It works blind

Nobody should need to read what it removes. The log (`cfgstrip-log.jsonl`)
records only *where* each cut happened: file, line range, enclosing item, the
kind and name of what was removed, line count, and the predicate. It never
records the removed text. A test enforces this.

Line numbers in the log refer to the file before stripping (pass 1).
Occasionally a second pass cleans up something the first pass exposed; those
records carry `"pass": 2` and refer to the file after pass 1.

## Usage

```sh
uv run --project tools/cfgstrip cfgstrip path/to/backend --dry-run --log plan.jsonl   # log only
uv run --project tools/cfgstrip cfgstrip path/to/backend --log strip.jsonl            # rewrite in place
uv run --project tools/cfgstrip cfgstrip path/to/backend --off enterprise --off private  # custom set
```

The default off set is `enterprise`, `private` and `enterprise_saml`. **Run it on a
copy or a fresh checkout**, since it rewrites files in place.

## Rewriting history

`history.py` (`cfgstrip-history`) applies the same stripping to every commit of
a git history, keeping authors, dates, messages and merges, and adding a
`Filtered-by: cfgstrip <version>` trailer. It's incremental with no state:
commits already present on the rewritten branch are recognised by their
metadata, and only new ones are rewritten. `tools/import-upstream` and the
`import-upstream` workflow drive it.

## Flags for review

Anything the tool can't handle confidently is flagged in the log, never guessed at:

- `cfg!` buried inside another macro's arguments (e.g. inside `json!`);
- predicates it can't parse;
- tree-sitter parse-error regions;
- optional dependencies that no feature enables any more (informational);
- features that used to switch on an off feature and are now untested combinations (informational).

## Tests

```sh
uv run --project tools/cfgstrip pytest
```
