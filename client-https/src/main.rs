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
    // Prepare the TLS client config
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let tls = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    // Prepare the HTTPS connector
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(tls)
        .https_or_http()
        .enable_http1()
        .build();

    let client = Client::builder().build::<_, hyper::Body>(https);

    let res = client.get(url).await?;

    println!("Response: {}", res.status());
    println!("Headers: {:#?}\n", res.headers());

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    println!("{}", String::from_utf8(body.into()).unwrap());

    println!("\n\nDone!");

    Ok(())
}
