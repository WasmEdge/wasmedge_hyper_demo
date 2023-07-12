use hyper::Request;
use tokio::net::TcpStream;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let url = "https://httpbin.org/get?msg=WasmEdge".parse::<hyper::Uri>().unwrap();
    fetch_https_url(url).await.unwrap();
}

async fn fetch_https_url(url: hyper::Uri) -> Result<()> {
    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(443);
    let addr = format!("{}:{}", host, port);
    let stream = TcpStream::connect(addr).await.unwrap();

    let config = wasmedge_rustls_api::ClientConfig::default();
    let tls_stream = wasmedge_rustls_api::stream::async_stream::TlsStream::connect(
        &config,
        url.host().unwrap(),
        stream,
    )
    .await
    .unwrap();

    let handshake =
        wasmedge_rustls_api::stream::async_stream::MidHandshake::Handshaking(tls_stream);
    let tls_stream = handshake.await.unwrap();

    let (mut sender, conn) = hyper::client::conn::handshake(tls_stream).await.unwrap();
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let authority = url.authority().unwrap().clone();

    let req = Request::builder()
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(hyper::Body::empty())?;

    let res = sender.send_request(req).await.unwrap();

    println!("Response: {}", res.status());
    println!("Headers: {:#?}\n", res.headers());

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    println!("{}", String::from_utf8(body.into()).unwrap());

    println!("\n\nDone!");

    Ok(())
}
