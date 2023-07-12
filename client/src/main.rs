#![deny(warnings)]
#![warn(rust_2018_idioms)]
use hyper::{body::HttpBody as _, Client};
use hyper::{Body, Method, Request};
// use tokio::io::{self, AsyncWriteExt as _};

// A simple type alias so as to DRY.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let url_str = "http://eu.httpbin.org/get?msg=Hello";
    println!("\nGET as byte stream: {}", url_str);
    let url = url_str.parse::<hyper::Uri>().unwrap();
    if url.scheme_str() != Some("http") {
        println!("This example only works with 'http' URLs.");
        return Ok(());
    }
    fetch_url(url).await?;
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let url_str = "http://eu.httpbin.org/get?msg=WasmEdge";
    println!("\nGET and get result as string: {}", url_str);
    let url = url_str.parse::<hyper::Uri>().unwrap();
    fetch_url_return_str(url).await?;
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let url_str = "http://eu.httpbin.org/post";
    let post_body_str = "hello wasmedge";
    println!("\nPOST and get result as string: {}", url_str);
    println!("with a POST body: {}", post_body_str);
    let url = url_str.parse::<hyper::Uri>().unwrap();
    post_url_return_str(url, post_body_str.as_bytes()).await
}

async fn fetch_url(url: hyper::Uri) -> Result<()> {
    let client = Client::new();
    let mut res = client.get(url).await?;

    println!("Response: {}", res.status());
    println!("Headers: {:#?}\n", res.headers());

    // Stream the body, writing each chunk to stdout as we get it
    // (instead of buffering and printing at the end).
    while let Some(next) = res.data().await {
        let chunk = next?;
        println!("{:#?}", chunk);
        // io::stdout().write_all(&chunk).await?;
    }

    Ok(())
}

async fn fetch_url_return_str(url: hyper::Uri) -> Result<()> {
    let client = Client::new();
    let mut res = client.get(url).await?;

    let mut resp_data = Vec::new();
    while let Some(next) = res.data().await {
        let chunk = next?;
        resp_data.extend_from_slice(&chunk);
    }
    println!("{}", String::from_utf8_lossy(&resp_data));

    Ok(())
}

async fn post_url_return_str(url: hyper::Uri, post_body: &'static [u8]) -> Result<()> {
    let client = Client::new();
    let req = Request::builder()
        .method(Method::POST)
        .uri(url)
        .body(Body::from(post_body))?;
    let mut res = client.request(req).await?;

    let mut resp_data = Vec::new();
    while let Some(next) = res.data().await {
        let chunk = next?;
        resp_data.extend_from_slice(&chunk);
    }
    println!("{}", String::from_utf8_lossy(&resp_data));

    Ok(())
}
