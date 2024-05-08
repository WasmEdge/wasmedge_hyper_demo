use bytes::Bytes;
use futures_util::StreamExt;

use axum::{extract::BodyStream, routing::get, routing::post, Router};
use tokio::net::TcpListener;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // build our application with a route
    let app = Router::new()
        .route("/", get(help))
        .route("/echo", post(echo));

    // run it
    let addr = "0.0.0.0:8080";
    let tcp_listener = TcpListener::bind(addr).await.unwrap();
    println!("listening on {}", addr);
    axum::Server::from_tcp(tcp_listener.into_std().unwrap())
        .unwrap()
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn help() -> &'static str {
    "Try POSTing data to /echo such as: `curl localhost:8080/echo -XPOST -d 'hello world'`\n"
}

async fn echo(mut stream: BodyStream) -> Bytes {
    if let Some(Ok(s)) = stream.next().await {
        s
    } else {
        Bytes::new()
    }
}
