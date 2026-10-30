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

## crates/crabbage-domain, crates/crabbage-store, crates/crabbage-import

Each crate is a library with only a crate doc (`src/lib.rs`) so far. Their responsibilities:

| Crate | Responsibility |
| --- | --- |
| `crabbage-domain` | Pure recipe rules. No IO |
| `crabbage-store` | SQLite persistence |
| `crabbage-import` | Recipe import from web pages, photos and Mealie |

## crates/crabbage

A Leptos 0.8 app. The `ssr` feature builds the server binary and the server-rendered router. The `hydrate` feature builds the WASM library that hydrates the page in the browser. The two features are independent and are never enabled together by the build.

| Module | Responsibility |
| --- | --- |
| `src/lib.rs` | Crate doc, module declarations, re-exports, and the `hydrate` entry point for the browser |
| `src/app.rs` | `App` component (title and heading) and `shell`, the HTML document around it |
| `src/server.rs` (`ssr`) | `router`: `/healthz`, the Leptos routes rendered with `shell`, and the site files as fallback |
| `src/main.rs` (`ssr`) | Server binary: reads the Leptos configuration and serves `router` on `site_addr`. Without `ssr` it is an empty stub |
| `tests/server.rs` | Requests against `router`: health check and the rendered title |
| `tests/binary.rs` | Runs the server binary: invalid `LEPTOS_SITE_ADDR` and an address already in use both exit with an error |

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

All lints are configured once in the workspace `Cargo.toml` and inherited with `[lints] workspace = true`. Clippy `pedantic`, `nursery` and `cargo` are denied, plus selected `restriction` lints (no `unwrap`, `expect`, `panic`, printing macros, indexing). Tests may `unwrap` (see `clippy.toml`). Use `#[expect(lint, reason = "...")]` for justified exceptions; `#[allow]` is denied. Duplicate crate versions that Leptos pulls in are allowed by name in `clippy.toml` (`allowed-duplicate-crates`) and by version in `deny.toml` (`bans.skip`), each with a reason.
