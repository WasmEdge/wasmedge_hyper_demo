# HTTP server example

## Quickstart with Docker

With an WASI-enabled Docker, you just need one line of command to build and run the HTTP server example. For details, refer to the [Dockerfile](../Dockerfile) and [docker-compose.yml](../docker-compose.yml) files.

```bash
docker compose build
docker compose run --no-TTY -p 8080:8080 server
```

Next, you can jump directly to the [Test](#test) section. If you want to build and run the application step by step on your own system, read on.

## Build

```bash
cargo build --target wasm32-wasi --release
```

## Run

```bash
wasmedge target/wasm32-wasi/release/wasmedge_hyper_server.wasm
```

## Test

Run the following from another terminal.

```bash
$ curl http://localhost:8080/echo -X POST -d "WasmEdge"
WasmEdge
```
