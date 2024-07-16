# HTTP server example using the axum crate

## Build

```bash
cargo build --target wasm32-wasi --release
```

## Run

```bash
wasmedge target/wasm32-wasi/release/wasmedge_axum_server.wasm
```

## Test

Run the following from another terminal.

```bash
$ curl http://localhost:8080/
Try POSTing data to /echo such as: `curl localhost:8080/echo -XPOST -d 'hello world'`
```

```bash
$ curl http://localhost:8080/echo -X POST -d "WasmEdge"
WasmEdge
```
