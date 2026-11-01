# Each platform is built on a runner of that platform, so no emulated compiler runs. Every
# downloaded tool is pinned and verified by SHA-256.

FROM rust:1.99.1-trixie@sha256:09027f873d45b8ecd1dc86f1fa13b7a62b875c961f017a0e331267b350430137 AS tools
ARG CARGO_LEPTOS_VERSION=0.3.11
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends musl-tools; \
    rm -rf /var/lib/apt/lists/*; \
    case "$(uname -m)" in \
        x86_64) leptos_sha=1c9cf0e1e3ffae6c8b58d29281d949dab560cea613b80d361f323f93071df7c8 ;; \
        aarch64) leptos_sha=8aa36d841d69fb62fa00a3e0d8670a52dc16a9a94c2c6f96c545d889c49f18aa ;; \
        *) echo "unsupported architecture: $(uname -m)" >&2; exit 1 ;; \
    esac; \
    curl -fsSL -o /tmp/leptos.tar.gz "https://github.com/leptos-rs/cargo-leptos/releases/download/v${CARGO_LEPTOS_VERSION}/cargo-leptos-$(uname -m)-unknown-linux-gnu.tar.gz"; \
    echo "$leptos_sha  /tmp/leptos.tar.gz" | sha256sum -c -; \
    tar -xzf /tmp/leptos.tar.gz -C /usr/local/bin --strip-components=1 --wildcards '*/cargo-leptos'; \
    rm -f /tmp/leptos.tar.gz; \
    cargo leptos --version

FROM tools AS site
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/src/target,id=site-target \
    cargo leptos build --release --frontend-only --cargo-locked \
    && install -d /out \
    && cp -r target/site /out/site \
    && cp target/release/hash.txt /out/hash.txt

FROM tools AS server
ARG TARGETARCH
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/src/target,id=server-target-${TARGETARCH} \
    set -eux; \
    triple="$(uname -m)-unknown-linux-musl"; \
    rustup target add "$triple"; \
    cargo build --release --locked --package crabbage --bin crabbage --features ssr --no-default-features --target "$triple"; \
    install -D -m 0555 "target/$triple/release/crabbage" /out/crabbage

FROM tools AS data
RUN install -d -o 65532 -g 65532 -m 0700 /data

FROM scratch
COPY --from=data --chown=65532:65532 /data /data
COPY --from=site /out/site /site
COPY --from=site /out/hash.txt /hash.txt
COPY --from=server /out/crabbage /crabbage
ENV CRABBAGE_DATA_DIR=/data \
    LEPTOS_SITE_ROOT=/site \
    LEPTOS_OUTPUT_NAME=crabbage \
    LEPTOS_HASH_FILES=true \
    LEPTOS_SITE_ADDR=0.0.0.0:3000
EXPOSE 3000
VOLUME /data
USER 65532:65532
ENTRYPOINT ["/crabbage"]
HEALTHCHECK CMD ["/crabbage", "--healthcheck"]
