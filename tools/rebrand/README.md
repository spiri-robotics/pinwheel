# rebrand

Renames the product in the frontend's user-visible text, from `Windmill` to
`Pinwheel`, and stops the UI linking to upstream's website. Like `cfgstrip`, it
parses the source with tree-sitter (the Svelte and TypeScript grammars), so it
only touches text a user can see:

| Rewritten | Left alone |
|---|---|
| markup text, attribute values (`title`, `label`, `tooltip`, …) | comments, `<style>`, identifiers, imports |
| string and template literal contents, in `<script>` and `{…}` expressions | object keys, `value`/`id`/`type`/… properties, comparisons, `switch` cases, literal types |
| | arguments to `includes`, `getItem`, `querySelector`, … ; regex literals |
| | `id`, `class`, `name`, `bind:`, `on:`, … attributes (links in them are still rewritten) |
| | tests, generated files (`*.gen.ts`, `src/lib/gen`), untracked build output |

Inside that text:

- **The name** in prose becomes `Pinwheel`/`pinwheel`/`PINWHEEL`. It must stand
  as a word, so `windmill-client`, `X-Windmill-Deploy-Origin`, `u/windmill/…` and
  `windmill.dev` don't match. Lowercase is renamed only inside text that contains
  a space, since a bare `'windmill'` is almost always a value (the HTTP route
  auth method, the Vault mount path, a postMessage tag).
- **Doc links** (`www.windmill.dev/docs`, `/blog`, `/changelog`, …) point at the
  fork's repository (changelog links at its releases), so nothing implies we're
  affiliated with upstream.
- **Everything else is flagged**, not rewritten: functional endpoints (Hub,
  telemetry, cloud host checks, `github.com/windmill-labs`), upstream emails
  (including the seeded `admin@windmill.dev`, which comes from the backend),
  pricing, terms and privacy links, mentions of upstream products ("Windmill Hub",
  "Windmill Cloud", "Windmill Enterprise"), and the name in a position the
  tool won't change. These need a person: reword, remove, or make configurable.

## Usage

```sh
uv run --project tools/rebrand rebrand frontend            # rewrite in place
uv run --project tools/rebrand rebrand frontend --check    # CI: change nothing, fail if anything is pending
uv run --project tools/rebrand rebrand frontend --log rebrand.jsonl   # every rewrite and flag, with lines
```

The tool is idempotent: run it after merging an upstream import and commit the result.

## Silencing flags

`known-flags.txt` lists the acknowledged flags, one line per occurrence:
file, matched text, reason. It has no line numbers, so code moving around
doesn't disturb it. `--check` fails only on flags missing from it. After fixing
a flag, or to accept the current set, run `--update-known`. The tool also
reports listed flags that no longer occur. Flags aren't silenced with inline
comments, because those would conflict with upstream changes on every import.

## Not covered yet

- `system_prompts/` (the AI assistant's prompts, shared with the CLI and
  generated into `prompts.ts`) still say Windmill.
- The backend's user-facing strings (emails, error messages).
- The logo (`WindmillIcon`) and favicons.

## Tests

```sh
uv run --project tools/rebrand pytest tools/rebrand
```
