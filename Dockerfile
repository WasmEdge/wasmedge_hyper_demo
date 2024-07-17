FROM --platform=$BUILDPLATFORM rust:1.64 AS buildbase
RUN rustup target add wasm32-wasi
WORKDIR /src

FROM --platform=$BUILDPLATFORM buildbase AS buildclient
COPY client/ /src
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,target=/usr/local/cargo/registry/index \
    RUSTFLAGS="--cfg wasmedge --cfg tokio_unstable" cargo build --target wasm32-wasi --release

FROM --platform=$BUILDPLATFORM buildbase AS buildserver
COPY server/ /src
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,target=/usr/local/cargo/registry/index \
    RUSTFLAGS="--cfg wasmedge --cfg tokio_unstable" cargo build --target wasm32-wasi --release

FROM --platform=$BUILDPLATFORM buildbase AS buildserverwarp
COPY server-axum/ /src
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,target=/usr/local/cargo/registry/index \
    RUSTFLAGS="--cfg wasmedge --cfg tokio_unstable" cargo build --target wasm32-wasi --release

FROM scratch AS client
ENTRYPOINT [ "wasmedge_hyper_client.wasm" ]
COPY --link --from=buildclient /src/target/wasm32-wasi/release/wasmedge_hyper_client.wasm  wasmedge_hyper_client.wasm

FROM scratch AS server
ENTRYPOINT [ "wasmedge_hyper_server.wasm" ]
COPY --link --from=buildserver /src/target/wasm32-wasi/release/wasmedge_hyper_server.wasm wasmedge_hyper_server.wasm

FROM scratch AS server-warp
ENTRYPOINT [ "wasmedge_warp_server.wasm" ]
COPY --link --from=buildserverwarp /src/target/wasm32-wasi/release/wasmedge_axum_server.wasm wasmedge_axum_server.wasm
