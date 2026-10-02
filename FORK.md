# Hand-off: the Windmill AGPL fork

## Background

Windmill ships some code inside AGPLv3-licensed files behind `enterprise` and
`private` compile flags, and its LICENSE file describes that code as
proprietary. We've stripped it from every commit in the fork, purely out of an
abundance of caution. Removing it isn't an acknowledgement that the code is
proprietary, and it shouldn't be read as one.

## What you're given

- This repository: Windmill's full history, with every commit mechanically
  filtered by `cfgstrip` so that no commit contains `enterprise`, `private` or
  `enterprise_saml` gated code. Authors, dates, messages and merges are kept;
  each filtered commit carries a `Filtered-by: cfgstrip <version>` trailer. The
  history shares no commits with upstream and none of the removed code appears
  anywhere in it, so working from `log`, `blame` and `bisect` keeps the fork's
  versions of removed features clean-room reimplementations. The fork's own work starts on top of upstream commit
  `fee401f01eebe411675a19eeff07b6f5435660b1` (release 1.821.0).
- `tools/cfgstrip/`: the tool that did the removal, with its README and tests.
- `tools/cfgstrip/logs/strip-fee401f.jsonl`: the removal log. It lists *where* code was cut (file, line,
  enclosing item, the item's kind and name, line count, predicate) and never
  the removed text. Treat it as your map of the extension points.

## Ground rules

- **Work only from the stripped tree**, Windmill's public documentation, and
  its OpenAPI spec (`backend/windmill-api/openapi.yaml`, Apache-2.0).
- **Don't consult upstream Windmill source** for anything the log shows as
  removed: not the GitHub repo, its history, diffs against it, or the
  Community Edition binaries. If you need behaviour that was removed, design it
  from the surrounding AGPL code, the public docs, and the log's one-line
  location data.
- **Don't decompile or disassemble any Windmill-distributed binary.**
- The `_oss.rs` placeholder files are AGPLv3 and are fair game. Fill their
  function bodies with your own implementations.

## Expected state

- `cargo check --features oss` and `cargo check --features oss --tests`
  both pass on the stripped tree (see `VERIFY.md`). Upstream's `oss` bundle
  never enabled the stripped flags, so behaviour should match an upstream
  `oss` build.
- **Build environment:** `nix-shell shell.nix` at the repo root provides
  clang, mold, pkg-config, openssl, krb5, protobuf, cmake and perl, and sets
  `SQLX_OFFLINE=true` and `LIBGSSAPI_IMPL=mit`. Then run
  `cd backend && cargo check --features oss`.
- `SQLX_OFFLINE=true` is required to build, using the committed `.sqlx`
  cache. Regenerate it later with `cargo sqlx prepare`, without the removed
  features.
- **16 compiler warnings** appear beyond upstream's: variables, imports and
  helper functions that only the removed code used (mostly in
  `windmill-api-workspaces/src/remote_deploy.rs` and `workspaces.rs`). This is
  ordinary AGPL code, so follow the compiler and delete or underscore them.
- **Upstream's agent instruction files are filtered out of every commit**
  (`AGENTS.md`, `CLAUDE.md`, `.claude/`, `.agents/`); they direct agents to work
  in Windmill's private repository. The fork's own `AGENTS.md` and
  `.claude/settings.json` replace them; the settings deny fetching or cloning
  from `windmill-labs`.

## Known leftovers

1. **Five comments** still mention upstream private module names
   (`oidc_ee`, `workspace_fairness_ee`, `jobs_ee`, `otel_ee`). They're comments
   only, with no code. Reword or delete them:
   - `backend/windmill-api-settings/src/lib.rs` (doc comment near `get_jwks`)
   - `backend/windmill-common/src/worker.rs` (doc comment near the fairness settings)
   - `backend/windmill-queue/tests/native_retry_test.rs` (two comments)
   - `backend/windmill-worker/src/worker.rs` (comment near the OTel span parent)
2. **Feature bundles** that used to switch on the removed flags (`ee`,
   `ee_core`, `ce_core`, `worker_windows_core`, `tantivy`, `all_sqlx_features`,
   and one in `windmill-git-sync`) are listed as flags in the log. Delete the
   `ee*`/`ce*` bundles. Keep `all_sqlx_features` and `tantivy` only if they
   build.
3. **29 optional dependencies** are no longer enabled by any feature (flagged in
   the log). They're harmless. Prune them when convenient.
4. **CI workflows, Dockerfiles and agent/skill docs** still reference the
   private repo and the EE build. None of it is needed. Replace them with a
   build of `--features oss` (plus whatever you add).
5. **The frontend is untouched.** It still contains license checks such as
   `$enterpriseLicense` gating and "(requires ee)" labels. A separate pass will
   come later.

## Work items, in order

1. **Verify:** `cargo check --features oss`, then the backend tests, then a
   smoke run (login with the seeded admin, run a script, run a flow).
2. **Remove the caps:** delete `check_nb_of_user`
   (`windmill-api/src/oauth2_oss.rs`), `_check_nb_of_groups`
   (`windmill-api-groups/src/groups.rs`) and `_check_nb_of_workspaces`
   (`windmill-api-workspaces/src/workspaces.rs`), plus their call sites.
3. **User management:** implement `create_user`, `set_password` and
   `hash_password` in `windmill-api/src/users_oss.rs` (argon2, which is
   already a dependency). Make `set_password` refuse LDAP-backed users.
4. **lldap login,** in `login()` in `windmill-api-users/src/users.rs`:
   - Bind against the configured lldap: search by `mail` with a read-only
     service account, then bind as the user's DN.
   - On success, upsert the `password` row with `login_type = 'ldap'` and call
     `create_session_token()`.
   - Sync groups on every login: `windmill_admins` → `super_admin`;
     `windmill_<ws>` → membership in workspace `<ws>`;
     `windmill_<ws>_<group>` → membership in group `<group>`. Remove
     memberships that are no longer present.
   - Take configuration (URL, base DN, bind DN/password, group prefix) from
     environment variables.
5. **Email:** implement `send_email*` in `windmill-common/src/email_oss.rs`
   with `lettre`, configured from environment variables.
6. **Audit log (optional):** implement `audit_log()` in
   `windmill-audit/src/audit_oss.rs` as a plain insert.
7. **Rename and rebrand:** new name and logo, remove Windmill's terms-of-service
   line from the login page, and add an AGPL source link in the footer.

## Pulling in upstream releases

The `import-upstream` workflow runs daily (or on demand from the Actions tab).
It rewrites any new upstream commits with the current `tools/cfgstrip`, pushes
them to `upstream-stripped`, and opens a PR into `main` (or adds them to the
one already open). Merge that PR with a merge commit, not a squash or rebase,
so later imports keep a common base. `tools/import-upstream` does the same by
hand. The workflow re-enables itself on every run, so GitHub's 60-day pause on
idle scheduled workflows doesn't stop it.

No state is kept anywhere. Rewriting preserves each commit's author, dates and
message, so the importer works out which upstream commits `upstream-stripped`
already covers by matching them, and only rewrites the new ones. The unstripped
upstream clone it uses is a temporary directory, deleted afterwards.

- **Rebrand after merging.** Upstream text arrives saying Windmill and linking
  to windmill.dev; the `no-upstream-branding` check fails until
  `uv run --project tools/rebrand rebrand frontend` has been run and committed
  (see `tools/rebrand/README.md`).
- **Never merge upstream directly** (`git merge` from upstream, GitHub's "Sync
  fork"). The rewritten history shares no commits with upstream, so git refuses
  such a merge as "unrelated histories", and the `no-gated-code` CI check fails
  if gated code appears anyway.
- **If a later commit needs better stripping,** improve `tools/cfgstrip`, reset
  `upstream-stripped` to just before that commit, and run the import again:
  everything from that point on is rewritten with the improved tool. Commits
  before it keep the tool version named in their trailer.
