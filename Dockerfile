FROM rust:1.99.1-alpine@sha256:7e9e2d068a107a2adafd19930011178d14a78326053cd4c3a9f7ad3fd049557d AS build
WORKDIR /src
COPY . .
RUN cargo build --locked --release --package rust-template \
    && install -D -m 0555 target/release/rust-template /out/rust-template

FROM scratch
COPY --from=build /out/rust-template /rust-template
USER 65532:65532
ENTRYPOINT ["/rust-template"]
