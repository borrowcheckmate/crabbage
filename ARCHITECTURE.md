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
Dockerfile                minimal runtime image
```

## crates/crabbage-domain, crates/crabbage-import

Each crate is a library with only a crate doc (`src/lib.rs`) so far. Their responsibilities:

| Crate | Responsibility |
| --- | --- |
| `crabbage-domain` | Pure recipe rules. No IO |
| `crabbage-import` | Recipe import from web pages, photos and Mealie |

## crates/crabbage-store

SQLite via `sqlx` (bundled SQLite, tokio runtime, no TLS). `open(data_dir)` creates the directory and `crabbage.db` if missing, sets WAL journaling and foreign keys on every connection, and applies the embedded migrations. `StoreError` wraps the io, sqlx and migration errors.

| Module | Responsibility |
| --- | --- |
| `src/lib.rs` | Crate doc, module declarations, re-exports |
| `src/pool.rs` | `open`: connection options, the embedded `MIGRATOR` |
| `src/error.rs` | `StoreError` |
| `src/users.rs` | `upsert_user`: one row per OIDC subject, returns the user id |
| `src/sessions.rs` | `create`, `find_user`, `delete`. Takes the raw token and stores only its SHA-256 hash |
| `src/login_attempts.rs` | `create` and `take` for pending logins. `take` deletes the row; attempts expire after 10 minutes |
| `migrations/` | SQL migrations, applied in file-name order |
| `tests/open.rs` | Fresh and existing databases, pragmas, nested data dir |
| `tests/users.rs`, `tests/sessions.rs`, `tests/login_attempts.rs` | Store behaviour; `tests/common` opens a migrated database in a temp dir |

Migrations:

| File | Tables |
| --- | --- |
| `0001_households_and_users.sql` | `households`, `users`, `user_households` |
| `0002_sessions.sql` | `login_attempts` (pending logins by `state`), `sessions` (by token hash, cascade on user delete) |

Migrations are embedded at compile time by `sqlx::migrate!()`. To add one, create `migrations/NNNN_description.sql` with the next number. Never edit an applied migration; add a new one. Tables are `STRICT`.

## crates/crabbage

A Leptos 0.8 app. The `ssr` feature builds the server binary and the server-rendered router. The `hydrate` feature builds the WASM library that hydrates the page in the browser. The two features are independent and are never enabled together by the build.

| Module | Responsibility |
| --- | --- |
| `src/lib.rs` | Crate doc, module declarations, re-exports, and the `hydrate` entry point for the browser |
| `src/app.rs` | `App` component (title and heading) and `shell`, the HTML document around it |
| `src/config/` (`ssr`) | Settings from environment variables: `Config::from_env` and `from_lookup`, `AiConfig`, `ConfigError` listing every problem. `mod.rs` re-exports; `settings.rs`, `vars.rs` (lookup and parsing), `error.rs`, `tests.rs` |
| `src/state.rs` (`ssr`) | `AppState`: leptos options, pool, HTTP client and the OIDC client. `FromRef` gives leptos_axum its `LeptosOptions` |
| `src/server.rs` (`ssr`) | `router(options, &Config, pool)`: discovers the provider, then builds `/healthz`, the `auth` routes, the Leptos routes and the site files as fallback. The session middleware wraps all of them |
| `src/auth/` (`ssr`) | Sign-in and sessions, see below. `mod.rs` declares and re-exports; `SessionUser` (the signed-in user id, set in request extensions) is re-exported from the crate root |
| `src/main.rs` (`ssr`) | Server binary: reads `Config` from the environment, opens the database in `data_dir` with `crabbage_store::open` (migrations run here), binds `site_addr`, then builds `router` (which does OIDC discovery) and serves it. Without `ssr` it is an empty stub |

### Sign-in (`src/auth/`)

| Module | Responsibility |
| --- | --- |
| `oidc.rs` | `Oidc`: discovery of `OIDC_ISSUER_URL`, the authorization request (code, PKCE S256, nonce, state, scopes `openid profile groups`), the code exchange and the ID token check. Name comes from `name`, then `preferred_username`, then the subject. The HTTP client does not follow redirects |
| `login.rs` | `GET /auth/login`: stores a login attempt, sets the `crabbage_login` cookie (the state, `Path=/auth/`, 10 minutes) and redirects (303) to the provider |
| `callback.rs` | `GET /auth/callback`: requires the `crabbage_login` cookie to equal `state` (otherwise 400, attempt kept), takes the attempt by `state` (unknown or expired: 400), exchanges the code, verifies the ID token and nonce (failure: 400), upserts the user, creates a session, sets the session cookie, clears the login cookie and redirects to `/` |
| `logout.rs` | `POST /auth/logout`: deletes the session row, clears the cookie, redirects to `/` |
| `middleware.rs` | `require_session`: every path except `/healthz`, `/auth/*` and `/pkg/*` needs an unexpired session, otherwise 303 to `/auth/login`. A valid session adds `SessionUser` to the request |
| `cookie.rs` | `Cookie` (name, path, max age) with two instances: `SESSION` (`crabbage_session`, `Path=/`, 30 days) and `LOGIN` (`crabbage_login`, `Path=/auth/`, 10 minutes). Each reads its value from a request and builds the `Set-Cookie` for set or clear. All are `HttpOnly; Secure; SameSite=Lax` |
| `session.rs` | Session lifetime, unix time, and the 32-byte base64url session token |
| `error.rs` | `AuthError`: 400 with a message for rejected logins, 500 for store or randomness failures |

Flow: the login attempt holds `state`, `nonce` and the PKCE verifier in `login_attempts`. The browser also holds the state in the `crabbage_login` cookie, so only the browser that started the login can complete it (login CSRF). The callback checks the cookie before it takes the attempt, then the attempt is deleted, and the ID token must match the stored nonce. The cookie holds the raw token; `sessions` stores only its SHA-256 hash.

### Tests

| Path | Responsibility |
| --- | --- |
| `tests/server/main.rs` | `healthz`, the rendered shell for a signed-in user, and the shared harness module declarations |
| `tests/server/login.rs` | Unauthenticated redirects, the authorization request, the callback (cookie attributes, unknown state, single use, wrong nonce) |
| `tests/server/session.rs` | Logout deletes the row and clears the cookie; expired and unknown sessions require a new login |
| `tests/server/common/` | `Harness`: the router on a temp database, and `issuer.rs`, a fake provider on 127.0.0.1 with a runtime-generated RSA key (discovery, JWKS, token endpoint). Its ID token carries the nonce the test sets |
| `tests/binary.rs` | Runs the server binary: missing config vars, invalid `LEPTOS_SITE_ADDR`, an address already in use and an unreachable issuer all exit with an error. The address is bound before discovery, so the bind tests stay offline. Each run that gets past config uses its own temp `CRABBAGE_DATA_DIR` |

## xtask

| Module | Responsibility |
| --- | --- |
| `cli.rs` | Command definitions (clap) |
| `task/plan.rs` | Maps a command to a list of actions. Composite commands (`ci`, `pre-commit`) live here |
| `task/run.rs` | Executes actions and reports violations |
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
