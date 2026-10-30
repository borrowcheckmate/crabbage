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

```sh
crabbage            # Hello, world!
crabbage Ada        # Hello, Ada!
```

Invalid names exit with code 2 and a message on stderr.

## Development

Open the repository in the devcontainer, or install the toolchain from `rust-toolchain.toml`.

```sh
cargo xtask --help       # all tasks
cargo xtask ci           # what CI runs
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the workflow and [ARCHITECTURE.md](ARCHITECTURE.md) for the layout.

## License

[EUPL-1.2](LICENSE)
