# Build stages run on the build platform. Only the server binary is cross-compiled for the target,
# so no emulated compiler runs. Every downloaded tool is pinned and verified by SHA-256.

FROM --platform=$BUILDPLATFORM rust:1.99.1-trixie@sha256:09027f873d45b8ecd1dc86f1fa13b7a62b875c961f017a0e331267b350430137 AS tools
ARG BUILDARCH
ARG ZIG_VERSION=0.15.2
ARG CARGO_ZIGBUILD_VERSION=0.24.0
ARG CARGO_LEPTOS_VERSION=0.3.11
RUN set -eux; \
    case "$BUILDARCH" in \
        amd64) \
            arch=x86_64; \
            zig_sha=02aa270f183da276e5b5920b1dac44a63f1a49e55050ebde3aecc9eb82f93239; \
            zigbuild_sha=3ca25ee4348f9abd0932ed4debe826d246e20fb616d0201de5bd05e7f965e9d9; \
            leptos_sha=1c9cf0e1e3ffae6c8b58d29281d949dab560cea613b80d361f323f93071df7c8 ;; \
        arm64) \
            arch=aarch64; \
            zig_sha=958ed7d1e00d0ea76590d27666efbf7a932281b3d7ba0c6b01b0ff26498f667f; \
            zigbuild_sha=de42626985f428033078dfc12ac8a59ee713f8e5627235ec883b1c899bee895c; \
            leptos_sha=8aa36d841d69fb62fa00a3e0d8670a52dc16a9a94c2c6f96c545d889c49f18aa ;; \
        *) echo "unsupported build platform: $BUILDARCH" >&2; exit 1 ;; \
    esac; \
    curl -fsSL -o /tmp/zig.tar.xz "https://ziglang.org/download/${ZIG_VERSION}/zig-${arch}-linux-${ZIG_VERSION}.tar.xz"; \
    echo "$zig_sha  /tmp/zig.tar.xz" | sha256sum -c -; \
    mkdir /opt/zig; \
    tar -xJf /tmp/zig.tar.xz -C /opt/zig --strip-components=1; \
    ln -s /opt/zig/zig /usr/local/bin/zig; \
    curl -fsSL -o /tmp/zigbuild.tar.xz "https://github.com/rust-cross/cargo-zigbuild/releases/download/v${CARGO_ZIGBUILD_VERSION}/cargo-zigbuild-${arch}-unknown-linux-gnu.tar.xz"; \
    echo "$zigbuild_sha  /tmp/zigbuild.tar.xz" | sha256sum -c -; \
    tar -xJf /tmp/zigbuild.tar.xz -C /usr/local/bin --strip-components=1 --wildcards '*/cargo-zigbuild'; \
    curl -fsSL -o /tmp/leptos.tar.gz "https://github.com/leptos-rs/cargo-leptos/releases/download/v${CARGO_LEPTOS_VERSION}/cargo-leptos-${arch}-unknown-linux-gnu.tar.gz"; \
    echo "$leptos_sha  /tmp/leptos.tar.gz" | sha256sum -c -; \
    tar -xzf /tmp/leptos.tar.gz -C /usr/local/bin --strip-components=1 --wildcards '*/cargo-leptos'; \
    rm -rf /tmp/*.tar* /tmp/*.tar.gz; \
    zig version; cargo-zigbuild --version; cargo leptos --version

FROM --platform=$BUILDPLATFORM tools AS site
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/src/target,id=site-target \
    cargo leptos build --release --frontend-only --cargo-locked \
    && install -d /out \
    && cp -r target/site /out/site

FROM --platform=$BUILDPLATFORM tools AS server
ARG TARGETARCH
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/src/target,id=server-target-${TARGETARCH} \
    set -eux; \
    case "$TARGETARCH" in \
        amd64) triple=x86_64-unknown-linux-musl ;; \
        arm64) triple=aarch64-unknown-linux-musl ;; \
        *) echo "unsupported target: $TARGETARCH" >&2; exit 1 ;; \
    esac; \
    rustup target add "$triple"; \
    cargo zigbuild --release --locked --package crabbage --bin crabbage --features ssr --no-default-features --target "$triple"; \
    install -D -m 0555 "target/$triple/release/crabbage" /out/crabbage

FROM --platform=$BUILDPLATFORM tools AS data
RUN install -d -o 65532 -g 65532 -m 0700 /data

FROM scratch
COPY --from=data --chown=65532:65532 /data /data
COPY --from=site /out/site /site
COPY --from=server /out/crabbage /crabbage
ENV CRABBAGE_DATA_DIR=/data \
    LEPTOS_SITE_ROOT=/site \
    LEPTOS_OUTPUT_NAME=crabbage \
    LEPTOS_SITE_ADDR=0.0.0.0:3000
EXPOSE 3000
VOLUME /data
USER 65532:65532
ENTRYPOINT ["/crabbage"]
