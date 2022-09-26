# HTTP client example

## Build

```
cargo build --target wasm32-wasi --release
```

## Run

```
$ wasmedge target/wasm32-wasi/release/wasmedge_hyper_client.wasm

GET as byte stream: http://eu.httpbin.org/get?msg=Hello
Response: 200 OK
Headers: {
    "date": "Mon, 26 Sep 2022 02:10:45 GMT",
    "content-type": "application/json",
    "content-length": "236",
    "connection": "keep-alive",
    "server": "gunicorn/19.9.0",
    "access-control-allow-origin": "*",
    "access-control-allow-credentials": "true",
}

b"{\n  \"args\": {\n    \"msg\": \"Hello\"\n  }, \n  \"headers\": {\n    \"Host\": \"eu.httpbin.org\", \n    \"X-Amzn-Trace-Id\": \"Root=1-63310a25-60b9d5683337106a7ceb7226\"\n  }, \n  \"origin\": \"13.87.135.123\", \n  \"url\": \"http://eu.httpbin.org/get?msg=Hello\"\n}\n"

GET as string: http://eu.httpbin.org/get?msg=WasmEdge
{
  "args": {
    "msg": "WasmEdge"
  },
  "headers": {
    "Host": "eu.httpbin.org",
    "X-Amzn-Trace-Id": "Root=1-63310a25-5a50726155ab22e62ecf7a25"
  },
  "origin": "13.87.135.123",
  "url": "http://eu.httpbin.org/get?msg=WasmEdge"
}
```

