# TFLite server example

## Prequsites

In order to run this example, you will first install Tensorflow Lite dependency libraries:

```
VERSION=0.12.1
curl -s -L -O --remote-name-all https://github.com/second-state/WasmEdge-tensorflow-deps/releases/download/$VERSION/WasmEdge-tensorflow-deps-TFLite-$VERSION-manylinux2014_x86_64.tar.gz
tar -zxf WasmEdge-tensorflow-deps-TFLite-$VERSION-manylinux2014_x86_64.tar.gz
rm -f WasmEdge-tensorflow-deps-TFLite-$VERSION-manylinux2014_x86_64.tar.gz
sudo mv libtensorflowlite_c.so /usr/local/lib
```

Then, install WasmEdge with Tensorflow Lite plugin:

```
curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install.sh | sudo bash -s -- -v $VERSION --plugins wasi_nn-tensorflowlite -p /usr/local
```

## Build

```
cargo build --target wasm32-wasi --release
```

## Run

```
wasmedge target/wasm32-wasi/release/wasmedge_hyper_server_tflite.wasm
```

## Test

Run the following from another terminal.

```
$ curl http://localhost:8080/classify -X POST --data-binary "@grace_hopper.jpg"
military uniform is detected with 206/255 confidence
```
