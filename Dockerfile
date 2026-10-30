FROM rust:1.99.1-alpine@sha256:7e9e2d068a107a2adafd19930011178d14a78326053cd4c3a9f7ad3fd049557d AS build
WORKDIR /src
COPY . .
RUN cargo build --locked --release --package crabbage --features ssr \
    && install -D -m 0555 target/release/crabbage /out/crabbage

FROM scratch
COPY --from=build /out/crabbage /crabbage
USER 65532:65532
ENTRYPOINT ["/crabbage"]
