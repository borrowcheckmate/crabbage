# crabbage

A Rust-based recipe book.

## Install

```sh
cargo install --git https://github.com/borrowcheckmate/crabbage crabbage
```

Or run the container image, published for `linux/amd64` and `linux/arm64`:

```sh
docker pull ghcr.io/borrowcheckmate/crabbage:main
```

`cargo xtask docker` builds the image locally and smoke-tests `/healthz` in a hardened container.

## Usage

Development server with live reload (needs [cargo-leptos](https://github.com/leptos-rs/cargo-leptos)):

```sh
cargo leptos watch
```

Release build of the server and the WASM bundle:

```sh
cargo leptos build --release
```

The server listens on `site-addr` from `[package.metadata.leptos]` in `crates/crabbage/Cargo.toml`, or on `LEPTOS_SITE_ADDR`. `GET /healthz` returns `ok` and needs no provider, so it answers even while the provider is unreachable. Sign-in returns `503 Service Unavailable` until the provider can be discovered, and each sign-in request tries again.

The server reads its settings from environment variables at startup and refuses to start if a required one is missing or invalid. The error names every problem.

| Variable | Required | Default | Meaning |
| --- | --- | --- | --- |
| `CRABBAGE_LANGUAGE` | no | `en` | App language: `en` or `nl` |
| `CRABBAGE_DATA_DIR` | no | `/data` | Directory for persistent data: `crabbage.db`, created and migrated at startup |
| `CRABBAGE_PUBLIC_URL` | yes | | Public `http(s)` URL the app is served at |
| `OIDC_ISSUER_URL` | yes | | `http(s)` issuer URL of the OpenID Connect provider |
| `OIDC_CLIENT_ID` | yes | | Client ID registered with the provider |
| `OIDC_CLIENT_SECRET` | yes | | Client secret for the provider (never printed) |
| `LEPTOS_SITE_ADDR` | no | `127.0.0.1:3000` | Address the server listens on. The image sets `0.0.0.0:3000` |
| `CRABBAGE_ADMIN_GROUP` | no | `admin` | Group whose members administer the app |
| `CRABBAGE_HOUSEHOLD_GROUP_PREFIX` | no | `crabbage-household-` | Prefix of the groups that name households |

Membership can come from groups or custom claims. Put users in the configured admin group and in groups named with the household prefix (the full group name identifies each household), or set `crabbage_role: "admin"` and `crabbage_household` (a string or list of strings) as custom claims on a Pocket ID group. Pocket ID merges group custom claims into members' ID token claims. If two groups set the same claim key, only one value arrives; use one group per household.
| `AI_BASE_URL` | with `AI_MODEL` | | `http(s)` base URL of an OpenAI-compatible API. Setting it turns AI on |
| `AI_MODEL` | with `AI_BASE_URL` | | Model to request. Setting it turns AI on |
| `AI_API_KEY` | no | | API key for the AI service, if it needs one (never printed) |
| `CRABBAGE_IMPORT_ALLOW_PRIVATE` | no | `false` | `true` or `false`: allow importing from private network addresses (e.g. a LAN recipe server) |

AI is enabled only when both `AI_BASE_URL` and `AI_MODEL` are set. Setting one without the other is an error. Empty values count as unset.

The image runs as user `65532` from `scratch`. It keeps the database in the `/data` volume, so mount a volume there. A bind mount must be writable by user `65532`. The image also sets `LEPTOS_SITE_ROOT=/site`, `LEPTOS_OUTPUT_NAME=crabbage` and `CRABBAGE_DATA_DIR=/data`.

### Docker Compose with Traefik

Traefik terminates TLS and forwards to port 3000. Replace `example.com` with your hostnames.

```yaml
services:
  crabbage:
    image: ghcr.io/borrowcheckmate/crabbage:main
    restart: unless-stopped
    read_only: true
    cap_drop: [ALL]
    security_opt: [no-new-privileges:true]
    healthcheck:
      test: ["CMD", "/crabbage", "--healthcheck"]
    environment:
      CRABBAGE_PUBLIC_URL: https://recipes.example.com
      OIDC_ISSUER_URL: https://id.example.com
      OIDC_CLIENT_ID: crabbage
      OIDC_CLIENT_SECRET: replace-me
    volumes:
      - crabbage-data:/data
    networks: [proxy]
    labels:
      traefik.enable: "true"
      traefik.docker.network: proxy
      traefik.http.routers.crabbage.rule: Host(`recipes.example.com`)
      traefik.http.routers.crabbage.entrypoints: websecure
      traefik.http.routers.crabbage.tls.certresolver: letsencrypt
      traefik.http.services.crabbage.loadbalancer.server.port: "3000"

volumes:
  crabbage-data:

networks:
  proxy:
    external: true
```

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
