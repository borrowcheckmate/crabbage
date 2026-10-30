# Architecture

Read this first. It tells you where things live so you only need to open the files relevant to your task. It describes the code as it is; planned work lives in issues.

## Layout

```
crates/crabbage-domain/   pure recipe rules, no IO
crates/crabbage-store/    SQLite persistence
crates/crabbage-import/   recipe import from web pages, photos and Mealie
crates/crabbage/          the Leptos app: library (SSR shell and hydration) + server binary
xtask/                    repository automation, invoked as `cargo xtask`
.githooks/                pre-commit and commit-msg hooks, both call xtask
.github/                  CI, release, dependabot, issue and PR templates
.devcontainer/            development container
Dockerfile                image build: native server build, WASM site, scratch runtime
```

## crates/crabbage-domain, crates/crabbage-import

Their responsibilities:

| Crate | Responsibility |
| --- | --- |
| `crabbage-domain` | Pure recipe rules. No IO |
| `crabbage-import` | Recipe import from web pages, photos and Mealie |

`crabbage-domain` modules:

| Module | Responsibility |
| --- | --- |
| `src/membership.rs` | `Role` (`Admin`, `Member`) and `Membership`, derived from a login's groups by `Membership::from_groups`: admin if the admin group is present, households are the groups with the household prefix and a name after it. `has_household_features` is true only with an active household |
| `tests/membership.rs` | Role, household matching, deduplication and the feature check |
| `src/quantity/mod.rs` | Declares the quantity submodules and re-exports `Dimension`, `Unit`, `Quantity` and `ImperialUnit` |
| `src/quantity/unit.rs` | `Dimension` (`Mass`, `Volume`, `Count`, `Unitless`) and `Unit`: `dimension`, `factor` to the base unit (g, ml or piece; `None` for unitless), `symbol` |
| `src/quantity/amount.rs` | `Quantity` (finite, non-negative amount and unit, `new` returns `None` otherwise): `convert_to` within a dimension, `checked_add` (same unit adds as it is, mixed units add in the base unit), `display` (g below 1000 g else kg; ml below 1000 ml else l; tsp and tbsp kept; count and unitless unchanged) |
| `src/quantity/imperial.rs` | `ImperialUnit` (oz, lb, cup, fl oz): `to_metric` gives the displayed metric quantity |
| `tests/quantity.rs` | Conversion within and across dimensions, spoons, display, addition, imperial input, unitless amounts and rejection of negative or non-finite amounts |
| `src/scaling.rs` | `Servings` (at least one, `new` returns `None` for zero) and `scale`: multiplies by the servings ratio, displays, then rounds per unit (g and ml by size, kg and l to 0.05, spoons to 0.25, pieces to 0.5). Non-zero amounts never round to zero; equal servings and unitless amounts are returned unchanged |
| `tests/scaling.rs` | Ratio scaling, per-unit rounding, the smallest step for tiny amounts, display boundary after rounding, identity at equal servings, unitless amounts, zero servings and overflow |
| `src/permissions.rs` | `can_manage_catalog(role)`: true only for `Role::Admin`. The catalog is ingredients, store sections and, later, categories |
| `tests/permissions.rs` | Admins manage the catalog, members do not |

## crates/crabbage-store

SQLite via `sqlx` (bundled SQLite, tokio runtime, no TLS). `open(data_dir)` creates the directory and `crabbage.db` if missing, sets WAL journaling and foreign keys on every connection, and applies the embedded migrations. `StoreError` wraps the io, sqlx and migration errors.

| Module | Responsibility |
| --- | --- |
| `src/lib.rs` | Crate doc, module declarations, re-exports |
| `src/pool.rs` | `open`: connection options, the embedded `MIGRATOR` |
| `src/error.rs` | `StoreError`: io, database and migration failures, `Conflict(text)` when a name or alias is taken or would equal a name or alias, and `InUse` when recipe lines use an ingredient |
| `src/users.rs` | `upsert_user`: one row per OIDC subject, returns the user id. `find`: the name and admin flag of a user (`User`) |
| `src/sessions.rs` | `create`, `find_user`, `delete`. Takes the raw token and stores only its SHA-256 hash |
| `src/login_attempts.rs` | `create` and `take` for pending logins. `take` deletes the row; attempts expire after 10 minutes |
| `src/households.rs` | `sync_membership` (one transaction per login: admin flag, households upserted by group name, memberships replaced, active household kept, cleared or defaulted to the first by group name), `households_of`, `active_household`, `set_active_household` (only for members) |
| `src/catalog/mod.rs` | Declares `ingredients` and `sections` (both public) and the shared rules in `names.rs` |
| `src/catalog/names.rs` | Maps unique violations to `Conflict`. A name is never an alias and an alias never a name, both case-insensitively; the checks run inside the store functions |
| `src/catalog/ingredients.rs` | `Ingredient` with its aliases. `list`, `list_pending`, `create`, `rename`, `delete` (`InUse` while recipe lines use it), `add_alias`, `remove_alias`, `set_section`, `approve`, `merge` (one transaction: recipe lines and aliases move to the target, the source name becomes an alias of it, the source is deleted). Missing rows return `false` |
| `src/catalog/sections.rs` | `Section`. `list` in position order, `create` at the end, `rename`, `delete` (ingredients keep their row and lose the section), `move_section` renumbers 0 to n-1 with the moved section at the new position |
| `migrations/` | SQL migrations, applied in file-name order |
| `tests/open.rs` | Fresh and existing databases, pragmas, nested data dir |
| `tests/users.rs`, `tests/sessions.rs`, `tests/login_attempts.rs`, `tests/households.rs` | Store behaviour; `tests/common` opens a migrated database in a temp dir |
| `tests/catalog_ingredients.rs`, `tests/catalog_aliases.rs`, `tests/catalog_merge.rs`, `tests/catalog_sections.rs` | Name and alias rules in every case, `InUse`, the review queue, merging (recipe lines inserted with SQL), section ordering and renumbering |

Migrations:

| File | Tables |
| --- | --- |
| `0001_households_and_users.sql` | `households`, `users`, `user_households` |
| `0002_sessions.sql` | `login_attempts` (pending logins by `state`), `sessions` (by token hash, cascade on user delete) |
| `0003_active_household.sql` | `users.active_household_id` (set null when the household is deleted) |
| `0004_catalog_and_recipes.sql` | `store_sections`, `ingredients` (`pending_review`), `ingredient_aliases` (cascade with the ingredient), `recipes`, `recipe_lines` (`ingredient_id` restricts deletes of used ingredients) |

Migrations are embedded at compile time by `sqlx::migrate!()`. To add one, create `migrations/NNNN_description.sql` with the next number. Never edit an applied migration; add a new one. Tables are `STRICT`.

## crates/crabbage

A Leptos 0.8 app. The `ssr` feature builds the server binary and the server-rendered router. The `hydrate` feature builds the WASM library that hydrates the page in the browser. The two features are independent and are never enabled together by the build.

| Module | Responsibility |
| --- | --- |
| `src/lib.rs` | Crate doc, module declarations, re-exports, and the `hydrate` entry point for the browser |
| `src/app.rs` | `App` component (title, heading and sign-out button, all from `t`) and `shell`, the HTML document around it. `shell` sets `<html lang>` from the render's language |
| `src/i18n/` | Translated text, see below. Compiled for both `ssr` and `hydrate` |
| `src/config/` (`ssr`) | Settings from environment variables: `Config::from_env` and `from_lookup`, `AiConfig`, `ConfigError` listing every problem. `CRABBAGE_LANGUAGE` is parsed into `i18n::Language`. `mod.rs` re-exports; `settings.rs`, `vars.rs` (lookup and parsing), `error.rs`, `tests.rs` |
| `src/state.rs` (`ssr`) | `AppState`: leptos options, pool, HTTP client, the configuration, the lazily discovered OIDC client (`oidc()`), the admin group and household prefix used on login, and the language. `FromRef` gives leptos_axum its `LeptosOptions` |
| `src/household.rs` (`ssr`) | `POST /household` (form field `household_id`): sets the signed-in user's active household. 303 to `/` on success, 403 when the user is not a member |
| `src/server.rs` (`ssr`) | `router(options, &Config, pool)`: builds `/healthz`, the `auth` routes, the `household` route, the `api` route, the Leptos routes and the site files as fallback. The Leptos routes provide the pool and the configured language as context, the error fallback the language. The session middleware wraps all of them |
| `src/api.rs` (`ssr`) | `POST /api/{*fn_name}`: serves every `#[server]` function, providing the pool and the language as context. Behind the session check |
| `src/access.rs` (`ssr`) | `signed_in` (the pool and the session user id), `require_admin` (the pool, or a 403 with the not-allowed message), `store_failure`, `clean` (trimmed, non-empty), `ensure_found` |
| `src/app.rs` | `App`: the `Router` with `/` (home, with admin links for admins), `/admin/ingredients`, `/admin/sections` and `/admin/review`, and the sign-out button. `shell` is the HTML document |
| `src/user.rs` | `current_user` server function: `SignedInUser` (name and `is_admin`) |
| `src/catalog/` | Catalog server functions and their records, see below. `types.rs` holds `IngredientRow` and `SectionRow` (both builds), `convert.rs` the store conversions (`ssr`) |
| `src/admin/` | The admin pages: `ingredients.rs`, `sections.rs`, `review.rs`, each showing a not-allowed message to non-admins, and `notice.rs` (failed actions) |
| `src/auth/` (`ssr`) | Sign-in and sessions, see below. `mod.rs` declares and re-exports; `SessionUser` (the signed-in user id, set in request extensions) is re-exported from the crate root |
| `src/main.rs` (`ssr`) | Server binary: reads `Config` from the environment, opens the database in `data_dir` with `crabbage_store::open` (migrations run here), binds `site_addr`, builds `router` and serves it. It starts without the provider. Without `ssr` it is an empty stub |

### Translations (`src/i18n/`, `locales/`)

`locales/en.json` and `locales/nl.json` are flat JSON objects of dotted keys to text (`"app.title": "crabbage"`), with `{name}` placeholders. Both are embedded with `include_str!`. They must have the same key set.

| Module | Responsibility |
| --- | --- |
| `src/i18n/mod.rs` | Module declarations and re-exports |
| `src/i18n/language.rs` | `Language` (`En`, `Nl`), re-exported from the crate root. `code` and `from_code` map to and from `en` and `nl` |
| `src/i18n/messages.rs` | `Messages`: one language's keys, parsed once per language from its embedded file. `get` returns the key when missing, `format` fills `{name}` placeholders and keeps unknown ones |
| `src/i18n/context.rs` | `t(key)` and `current_language()` read the language from leptos context, English when none is provided. `document_language()` (`hydrate`) reads the `lang` attribute of the page |
| `src/i18n/tests.rs` | Every file in `locales/` parses, has the English keys and a language code as its name; every key is translated; formatting; language codes |

The language flows from `Config` to `AppState`, then to the Leptos routes and the error fallback, which provide it as context for each render. `shell` sets `<html lang>` from it and `App` reads the text with `t`. The browser takes the language from the `lang` attribute of the page before it hydrates.

To add a language: add `locales/<code>.json` with the same keys, add a `Language` variant with its code in `from_code` and `code`, and add its arm in `Messages::for_language`, which embeds the file. Visible text in `view!` blocks must come from `t`; `tests/strings.rs` fails on string literals used as text children.

### Sign-in (`src/auth/`)

| Module | Responsibility |
| --- | --- |
| `oidc.rs` | `Oidc`: discovery of `OIDC_ISSUER_URL`, the authorization request (code, PKCE S256, nonce, state, scopes `openid profile groups`), the code exchange and the ID token check. Name comes from `name`, then `preferred_username`, then the subject. The `groups` claim is returned in `Identity`; a missing claim counts as no groups. The HTTP client does not follow redirects |
| `login.rs` | `GET /auth/login`: stores a login attempt, sets the `crabbage_login` cookie (the state, `Path=/auth/`, 10 minutes) and redirects (303) to the provider |
| `callback.rs` | `GET /auth/callback`: requires the `crabbage_login` cookie to equal `state` (otherwise 400, attempt kept), takes the attempt by `state` (unknown or expired: 400), exchanges the code, verifies the ID token and nonce (failure: 400), upserts the user, derives the membership from `groups` and syncs it (`households::sync_membership`), creates a session, sets the session cookie, clears the login cookie and redirects to `/` |
| `logout.rs` | `POST /auth/logout`: deletes the session row, clears the cookie, redirects to `/` |
| `middleware.rs` | `require_session`: every path except `/healthz`, `/auth/*` and `/pkg/*` needs an unexpired session, otherwise 303 to `/auth/login`. A valid session adds `SessionUser` to the request |
| `cookie.rs` | `Cookie` (name, path, max age) with two instances: `SESSION` (`crabbage_session`, `Path=/`, 30 days) and `LOGIN` (`crabbage_login`, `Path=/auth/`, 10 minutes). Each reads its value from a request and builds the `Set-Cookie` for set or clear. All are `HttpOnly; Secure; SameSite=Lax` |
| `session.rs` | Session lifetime, unix time, and the 32-byte base64url session token |
| `error.rs` | `AuthError`: 400 with a message for rejected logins, 500 for store or randomness failures, 503 when the provider cannot be discovered |

The provider is discovered by `AppState::oidc()` on the first request that needs it, and the client is kept only after discovery succeeds. While the provider is unreachable, `/auth/login` and `/auth/callback` answer 503 and the next request tries again. `/healthz` and the pages that need no session do not depend on the provider.

Flow: the login attempt holds `state`, `nonce` and the PKCE verifier in `login_attempts`. The browser also holds the state in the `crabbage_login` cookie, so only the browser that started the login can complete it (login CSRF). The callback checks the cookie before it takes the attempt, then the attempt is deleted, and the ID token must match the stored nonce. The cookie holds the raw token; `sessions` stores only its SHA-256 hash.

Roles and households follow the `groups` claim on every login: the callback replaces the user's admin flag and household memberships, so changes at the provider take effect on the next login. Each `crabbage-household-*` group is a household, created on first sight and shared by its members. The active household defaults to the first by group name, and a user keeps their choice across logins while they stay a member. Users without a household can use recipes; the shopping list and planner need an active household (`has_household_features`).

### Catalog and server functions

Admin pages call server functions. Each is a `#[server(prefix = "/api", endpoint = "...")]` function, so it is served at `/api/<endpoint>`. Input is form-encoded (`PostUrl`, the default) and output is JSON. Every function has an explicit `endpoint`, so the URL does not depend on the function name. The function name in PascalCase is the marker type that `ServerAction::<Marker>` uses in a page.

Each server function in `src/catalog/` begins with `crate::access::require_admin()`, which returns the pool. Errors go through `store_failure`, so each message is translated, and a missing row through `ensure_found`. A non-admin gets a 403 (`ResponseOptions` is set from the context) with the not-allowed text. The body of a server function is compiled for `ssr` only, so anything it needs from the store is imported inside the body. The client build keeps the signature and the record types, which must serialise on both sides.

To add an endpoint: add the store function with its tests in `crabbage-store`, add the `#[server]` function to the matching file in `src/catalog/` with `require_admin`, add a `ServerAction` and its form to the page in `src/admin/`, add the messages to both locale files and cover the endpoint in `tests/server/catalog.rs`.

Permissions: the session user's `is_admin` (set from the admin group on every login) becomes a `Role`, and `can_manage_catalog` decides. The server functions enforce the rule; the page only decides what it shows.

Pages load data in one `<Suspense>` whose closure returns a `Suspend` future that awaits the server functions. Pages do not create `Resource`s: `reactive_graph` keeps the current transition in a process-wide static that `leptos_router` sets around each route, so resources of concurrent server renders can register with another request's transition and never resolve.

### Tests

| Path | Responsibility |
| --- | --- |
| `tests/server/main.rs` | `healthz`, the rendered shell for a signed-in user in English and in Dutch (`lang`, title, sign-out text), and the shared harness module declarations |
| `tests/strings.rs` | No string literal is used as text in a `view!` block under `src/`. Tests the heuristic on inline samples |
| `tests/server/login.rs` | Unauthenticated redirects, the authorization request, the callback (cookie attributes, unknown state, single use, wrong nonce) |
| `tests/server/session.rs` | Logout deletes the row and clears the cookie; expired and unknown sessions require a new login |
| `tests/server/household.rs` | Group claims on login (admin, households, changes on the next login), switching the active household, 403 for non-members, a user without households |
| `tests/server/catalog.rs` | Admin create, rename and delete; members refused on every endpoint with 403; alias uniqueness through the API; sections created and reordered; the review queue; merge through the API |
| `tests/server/screenshots.rs` | Ignored. Writes the rendered pages to `CRABBAGE_SCREENSHOT_DIR` for review screenshots |
| `tests/server/pages.rs` | Admin pages render for admins, show the not-allowed message to members, and the home page links to them for admins only |
| `tests/server/common/` | `Harness`: the router on a temp database, and `issuer.rs`, a fake provider on 127.0.0.1 with a runtime-generated RSA key (discovery, JWKS, token endpoint). Its ID token carries the nonce and the `groups` claim the test sets |
| `tests/server/discovery.rs` | Lazy discovery: with an unreachable provider sign-in is 503, and once the provider listens on the same address it is 303 |
| `tests/binary.rs` | Runs the server binary: missing config vars and invalid `LEPTOS_SITE_ADDR` or an address already in use exit with an error. With an unreachable issuer the server runs: `/healthz` is 200 and `/auth/login` is 503. Each run uses its own temp `CRABBAGE_DATA_DIR`, and the server process is killed when the test ends |

## xtask

| Module | Responsibility |
| --- | --- |
| `cli.rs` | Command definitions (clap) |
| `task/plan.rs` | Maps a command to a list of actions. Composite commands (`ci`, `pre-commit`) live here |
| `task/run.rs` | Executes actions and reports violations |
| `task/smoke.rs` | Container smoke test for `cargo xtask docker`: the hardened `docker run` step, `/healthz` polling over a raw HTTP/1.1 `GET` (30 s), and the status-line parser. `run.rs` always runs `docker rm -f` afterwards |
| `task/step.rs` | An external process invocation |
| `policy/repository.rs` | Applies every file rule to the tracked files |
| `policy/comments.rs` | Finds plain comments in Rust source |
| `policy/module_root.rs` | Ensures `mod.rs` only declares and re-exports |
| `policy/length.rs` | Maximum lines per Rust file |
| `policy/layout.rs` | Allowed documents, forbidden files (`.env`, keys), size limit |
| `policy/leaks.rs` | Personal emails, home paths, tokens, image metadata, `authors` |
| `policy/identity.rs` | Anonymous git author and committer emails |
| `policy/commit.rs` | Conventional commits, no tool attribution |
| `repo.rs` | Git queries |

To add a check, add a submodule under `policy/`, call it from `repository.rs` (file rules) or `commit.rs` (message rules), and test it in the same file.

To add a task, add a variant to `cli.rs` and map it in `task/plan.rs`. Prefer composing existing steps.

## Container image

`Dockerfile` has five stages. Each platform is built natively on a runner of that platform (`ubuntu-24.04` for amd64, `ubuntu-24.04-arm` for arm64), so no emulated compiler runs:

| Stage | Contents |
| --- | --- |
| `tools` | Rust on Debian trixie, pinned by digest. `musl-tools` from apt, and cargo-leptos from its release archive, checked by SHA-256 |
| `site` | `cargo leptos build --release --frontend-only`: the WASM bundle, JS and CSS in `target/site`, copied to `/out/site` |
| `server` | `cargo build` of `crabbage` with `ssr` for the native musl target `$(uname -m)-unknown-linux-musl`. Bundled SQLite is C code compiled with `musl-gcc`; rustls needs no system libraries, so the binary is static |
| `data` | `/data` owned by 65532:65532 |
| final | `FROM scratch`. Copies `/data`, `/site` and `/crabbage`. Sets `CRABBAGE_DATA_DIR`, `LEPTOS_SITE_ROOT`, `LEPTOS_OUTPUT_NAME` and `LEPTOS_SITE_ADDR`. Runs as `65532:65532`, exposes 3000, declares `/data` as a volume |

The cargo-leptos version and its checksum are pinned in `ARG` and in the `tools` stage. CI's `image` job is a matrix with one leg per platform. Each leg builds its platform with its own GitHub Actions cache scope and pushes the image by digest on pushes to `main`. The `manifest` job then combines the digests into `ghcr.io/borrowcheckmate/crabbage:main` and `:sha-<short>`.

## Rules enforced by `cargo xtask policy`

- Rust files have at most 500 lines.
- `mod.rs` contains only `mod` declarations and `pub use` re-exports. `lib.rs` is exempt.
- No plain comments. Use `///` docs. `// NOTE:` is the escape hatch for the rare unavoidable comment.
- Documents live in `README.md`, `AGENTS.md`, `ARCHITECTURE.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CHANGELOG.md` or `.github/`.
- No `.env` files, keys, credentials, OS metadata files or files over 256 KiB.
- No personal emails, home directory paths, tokens or image metadata in any tracked file.
- No `authors` in `Cargo.toml`.

## Lints

All lints are configured once in the workspace `Cargo.toml` and inherited with `[lints] workspace = true`. Clippy `pedantic`, `nursery` and `cargo` are denied, plus selected `restriction` lints (no `unwrap`, `expect`, `panic`, printing macros, indexing). Tests may `unwrap` (see `clippy.toml`). Use `#[expect(lint, reason = "...")]` for justified exceptions; `#[allow]` is denied. Duplicate crate versions that Leptos and sqlx pull in are allowed by name in `clippy.toml` (`allowed-duplicate-crates`) and by version in `deny.toml` (`bans.skip`), each with a reason.
