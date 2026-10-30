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

## Development

Open the repository in the devcontainer, or install the toolchain from `rust-toolchain.toml`.

```sh
cargo xtask --help       # all tasks
cargo xtask ci           # what CI runs
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the workflow and [ARCHITECTURE.md](ARCHITECTURE.md) for the layout.

## License

[EUPL-1.2](LICENSE)
