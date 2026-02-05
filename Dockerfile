FROM --platform=$BUILDPLATFORM rust:1.83 AS buildbase
RUN rustup target add wasm32-wasip1
WORKDIR /src

FROM --platform=$BUILDPLATFORM buildbase AS buildclient
COPY client/ /src
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,target=/usr/local/cargo/registry/index \
    RUSTFLAGS="--cfg wasmedge --cfg tokio_unstable" cargo build --target wasm32-wasip1 --release

FROM --platform=$BUILDPLATFORM buildbase AS buildserver
COPY server/ /src
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,target=/usr/local/cargo/registry/index \
    RUSTFLAGS="--cfg wasmedge --cfg tokio_unstable" cargo build --target wasm32-wasip1 --release

FROM --platform=$BUILDPLATFORM buildbase AS buildserveraxum
COPY server-axum/ /src
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,target=/usr/local/cargo/registry/index \
    RUSTFLAGS="--cfg wasmedge --cfg tokio_unstable" cargo build --target wasm32-wasip1 --release

FROM scratch AS client
ENTRYPOINT [ "wasmedge_hyper_client.wasm" ]
COPY --link --from=buildclient /src/target/wasm32-wasip1/release/wasmedge_hyper_client.wasm wasmedge_hyper_client.wasm

FROM scratch AS server
ENTRYPOINT [ "wasmedge_hyper_server.wasm" ]
COPY --link --from=buildserver /src/target/wasm32-wasip1/release/wasmedge_hyper_server.wasm wasmedge_hyper_server.wasm

FROM scratch AS server-axum
ENTRYPOINT [ "wasmedge_axum_server.wasm" ]
COPY --link --from=buildserveraxum /src/target/wasm32-wasip1/release/wasmedge_axum_server.wasm wasmedge_axum_server.wasm
