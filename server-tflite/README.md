# TFLite server example

## Build

```
cargo build --target wasm32-wasi --release
```

## Run

```
wasmedge-tensorflow-lite target/wasm32-wasi/release/wasmedge_hyper_server_tflite.wasm
```

## Test

Run the following from another terminal.

```
$ curl http://localhost:3000/classify -X POST --data-binary "@grace_hopper.jpg"
military uniform is detected with 206/255 confidence
```
