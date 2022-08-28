# TFLite server example

## Build

```
cargo build --target wasm32-wasi --release
```

## Run

```
wasmedge target/wasm32-wasi/release/wasmedge_hyper_server.wasm
```

## Test

Run the following from another terminal.

```
$ curl http://localhost:3000/echo -X POST -d "WasmEdge"
WasmEdge
```
