FROM rust:1.99.1-slim-trixie@sha256:f6c4a35d21dbb3c89d7a8d056c193e94fc09037f2b58dd858b09c39a47dd02d7

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
