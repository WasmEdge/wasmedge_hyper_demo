use hyper::Client;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let url = "https://httpbin.org/get?msg=WasmEdge"
        .parse::<hyper::Uri>()
        .unwrap();
    fetch_https_url(url).await.unwrap();
}

async fn fetch_https_url(url: hyper::Uri) -> Result<()> {
    let https = wasmedge_hyper_rustls::connector::new_https_connector(
        wasmedge_rustls_api::ClientConfig::default(),
    );
    let client = Client::builder().build::<_, hyper::Body>(https);

    let res = client.get(url).await?;

    println!("Response: {}", res.status());
    println!("Headers: {:#?}\n", res.headers());

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    println!("{}", String::from_utf8(body.into()).unwrap());

    println!("\n\nDone!");

    Ok(())
}
