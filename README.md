# WasmEdge Hyper Demo

In this project, we demonstrate how to use **hyper** and **tokio** to build async HTTP client and server in WebAssembly and execute it using [WasmEdge](https://github.com/WasmEdge/WasmEdge).

## Why tokio in WasmEdge?

There are growing demands to perform network requests in WASM and cloud computing. But it would be inefficient to perform network requests synchronously so we need async in WASM.

As tokio is widely accepted, we can bring many projects that depend on tokio to WASM if we can port tokio into WASM. After that, the developers can have async functions in WASM as well as efficient programs.

With the help of tokio support of WasmEdge, the developers can compile the projects that use tokio into WASM and execute it using WasmEdge.

## Quickstart with Docker

The easiest way to get started is to use a version of Docker Desktop or Docker CLI with Wasm support.

* [Install Docker Desktop + Wasm (Beta)](https://docs.docker.com/desktop/wasm/)
* [Install Docker CLI + Wasm](https://github.com/chris-crone/wasm-day-na-22/tree/main/server)

Build all the examples using Docker. There is no need to install Rust or WasmEdge here since Docker sets it all up for you.

```bash
docker compose build
```

Then, you just need to type one command to run each example.

```bash
# To run the HTTP client example
$ docker compose run --no-TTY client
... ...

# To run the HTTP server
$ docker compose run --no-TTY -p 8080:8080 server
... ...
# Test the HTTP server using curl
$ curl http://localhost:8080/echo -X POST -d "WasmEdge"
WasmEdge

# To run the Axum HTTP server
$ docker compose run --no-TTY -p 8080:8080 server-axum
... ...
# Test the HTTP server using curl
$ curl http://localhost:8080/echo -X POST -d "WasmEdge"
WasmEdge
```

It runs both the client and server examples in this repo. See the [Dockerfile](Dockerfile) and [docker-compose.yml](docker-compose.yml) files. The [client example](client) will run and quit upon completion. The [server example](server) starts a server that listens for incoming HTTP requests, and you can interact with it through `curl`.

However, if you want to build and run the examples step by step on your own system, follow the detailed instructions below.

## Prerequisites

### Install Rust

Install Rust and add the `wasm32-wasip1` target:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install the WASM target
rustup target add wasm32-wasip1
```

**Note:** The target `wasm32-wasi` has been renamed to `wasm32-wasip1` in Rust 1.84+. If you're using an older Rust version, use `wasm32-wasi` instead.

### Install WasmEdge

Install the WasmEdge runtime:

```bash
curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install.sh | bash
source $HOME/.wasmedge/env
```

To run the [HTTP server with Tensorflow](server-tflite/README.md) example, install with all extensions:

```bash
curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install.sh | bash -s -- -e all
source $HOME/.wasmedge/env
```

## Project Configuration

### Cargo.toml Dependencies

This project uses patched versions of tokio, hyper, mio, and socket2 for WASI compatibility:

```toml
[patch.crates-io]
tokio = { git = "https://github.com/second-state/wasi_tokio.git", branch = "v1.36.x" }
socket2 = { git = "https://github.com/second-state/socket2.git", branch = "v0.5.x" }
hyper = { git = "https://github.com/second-state/wasi_hyper.git", branch = "v0.14.x" }
mio = { git = "https://github.com/second-state/wasi_mio.git", branch = "v0.8.x" }

[dependencies]
hyper = { version = "0.14", features = ["full"]}
tokio = { version = "1.36", features = ["rt", "macros", "net", "time", "io-util"]}
```

### Cargo Config

Create `.cargo/config.toml` in your project root:

```toml
[build]
target = "wasm32-wasip1"

[target.wasm32-wasip1]
runner = "wasmedge"
```

## Building

Build the project with:

```bash
cargo build --release
```

The output `.wasm` file will be in `target/wasm32-wasip1/release/`.

## Running

Run the compiled WebAssembly with WasmEdge:

```bash
wasmedge target/wasm32-wasip1/release/wasmedge_hyper_server.wasm
```

## Examples

Details about the example apps are as below.

* [HTTP client](client/README.md)
* [HTTP server](server/README.md)
* [HTTP server with Axum framework](server-axum/README.md)
* [HTTP server with Tensorflow](server-tflite/README.md)

## Version Compatibility

| Component | Version |
|-----------|---------|
| Rust | Latest stable |
| WasmEdge | Latest |
| Target | wasm32-wasip1 |
| tokio | 1.36.x (patched) |
| hyper | 0.14.x (patched) |
| mio | 0.8.x (patched) |
