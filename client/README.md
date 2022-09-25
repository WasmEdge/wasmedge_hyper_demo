# HTTP client example

## Build

```
cargo build --target wasm32-wasi --release
```

## Run

```
$ wasmedge target/wasm32-wasi/release/wasmedge_hyper_client.wasm

Response: 301 Moved Permanently
Headers: {
    "age": "26700",
    "cache-control": "public, max-age=0, must-revalidate",
    "content-length": "36",
    "content-type": "text/plain",
    "date": "Sun, 28 Aug 2022 09:43:53 GMT",
    "location": "https://wasmedge.org/",
    "server": "Netlify",
    "x-nf-request-id": "01GBJPTXGYP4GJJYP58YX5J975",
}
```

