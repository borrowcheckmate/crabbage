FROM rust:1.99.1-slim-trixie@sha256:b8e062b1c3a80183df83d9ba187cadd2b5ab84bc78d453dbfeb2fccd5f982247

RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates git \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy rustfmt llvm-tools-preview \
    && cargo install --locked cargo-deny@0.20.2 cargo-llvm-cov@0.9.1 \
    && rm -rf "$CARGO_HOME/registry"

RUN useradd --create-home --uid 1000 --shell /bin/bash dev \
    && mkdir -p "$CARGO_HOME/registry" \
    && chown -R dev:dev "$CARGO_HOME/registry"

USER dev
