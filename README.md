# crabbage

A Rust-based recipe book.

## Install

```sh
cargo install --git https://github.com/borrowcheckmate/crabbage crabbage
```

Or build the container image:

```sh
cargo xtask docker
```

## Usage

Development server with live reload (needs [cargo-leptos](https://github.com/leptos-rs/cargo-leptos)):

```sh
cargo leptos watch
```

Release build of the server and the WASM bundle:

```sh
cargo leptos build --release
```

The server listens on `site-addr` from `[package.metadata.leptos]` in `crates/crabbage/Cargo.toml`. `GET /healthz` returns `ok`.

The server reads its settings from environment variables at startup and refuses to start if a required one is missing or invalid. The error names every problem.

| Variable | Required | Default | Meaning |
| --- | --- | --- | --- |
| `CRABBAGE_LANGUAGE` | no | `en` | App language: `en` or `nl` |
| `CRABBAGE_DATA_DIR` | no | `/data` | Directory for persistent data: `crabbage.db`, created and migrated at startup |
| `CRABBAGE_PUBLIC_URL` | yes | | Public `http(s)` URL the app is served at |
| `OIDC_ISSUER_URL` | yes | | `http(s)` issuer URL of the OpenID Connect provider |
| `OIDC_CLIENT_ID` | yes | | Client ID registered with the provider |
| `OIDC_CLIENT_SECRET` | yes | | Client secret for the provider (never printed) |
| `CRABBAGE_ADMIN_GROUP` | no | `crabbage-admin` | Group whose members administer the app |
| `CRABBAGE_HOUSEHOLD_GROUP_PREFIX` | no | `crabbage-household-` | Prefix of the groups that name households |
| `AI_BASE_URL` | with `AI_MODEL` | | `http(s)` base URL of an OpenAI-compatible API. Setting it turns AI on |
| `AI_MODEL` | with `AI_BASE_URL` | | Model to request. Setting it turns AI on |
| `AI_API_KEY` | no | | API key for the AI service, if it needs one (never printed) |

AI is enabled only when both `AI_BASE_URL` and `AI_MODEL` are set. Setting one without the other is an error. Empty values count as unset.

Sign-in uses the OpenID Connect provider (Pocket ID). Register `{CRABBAGE_PUBLIC_URL}/auth/callback` as the redirect URI of the client. Every page except `/healthz` needs a session and redirects to `/auth/login` without one.

## Development

Open the repository in the devcontainer, or install the toolchain from `rust-toolchain.toml`.

```sh
cargo xtask --help       # all tasks
cargo xtask ci           # what CI runs
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the workflow and [ARCHITECTURE.md](ARCHITECTURE.md) for the layout.

## License

[EUPL-1.2](LICENSE)
