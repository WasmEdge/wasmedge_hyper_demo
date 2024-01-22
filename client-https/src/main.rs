#![deny(warnings)]
#![warn(rust_2018_idioms)]

// use tokio::io::{self, AsyncWriteExt as _};

use std::{
    os::fd::{FromRawFd, IntoRawFd},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use http_body_util::{BodyExt, Empty};
use hyper::{body::Bytes, Request};

use rustls::pki_types::ServerName;
use tokio::net::TcpStream;

type MainResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> MainResult<()> {
    pretty_env_logger::init();

    let url = "https://httpbin.org/get?msg=WasmEdge"
        .parse::<hyper::Uri>()
        .unwrap();
    fetch_https_url(url).await
}

use pin_project::pin_project;
use tokio_rustls::TlsConnector;

#[pin_project]
#[derive(Debug)]
struct TokioIo<T> {
    #[pin]
    inner: T,
}

impl<T> TokioIo<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    #[allow(dead_code)]
    pub fn inner(self) -> T {
        self.inner
    }
}

impl<T> hyper::rt::Read for TokioIo<T>
where
    T: tokio::io::AsyncRead,
{
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        mut buf: hyper::rt::ReadBufCursor<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let n = unsafe {
            let mut tbuf = tokio::io::ReadBuf::uninit(buf.as_mut());
            match tokio::io::AsyncRead::poll_read(self.project().inner, cx, &mut tbuf) {
                Poll::Ready(Ok(())) => tbuf.filled().len(),
                other => return other,
            }
        };

        unsafe {
            buf.advance(n);
        }
        Poll::Ready(Ok(()))
    }
}

impl<T> hyper::rt::Write for TokioIo<T>
where
    T: tokio::io::AsyncWrite,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        tokio::io::AsyncWrite::poll_write(self.project().inner, cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        tokio::io::AsyncWrite::poll_flush(self.project().inner, cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        tokio::io::AsyncWrite::poll_shutdown(self.project().inner, cx)
    }

    fn is_write_vectored(&self) -> bool {
        tokio::io::AsyncWrite::is_write_vectored(&self.inner)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> Poll<std::prelude::v1::Result<usize, std::io::Error>> {
        tokio::io::AsyncWrite::poll_write_vectored(self.project().inner, cx, bufs)
    }
}

async fn fetch_https_url(url: hyper::Uri) -> MainResult<()> {
    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(443);
    let addr = format!("{}:{}", host, port);

    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));
    let stream = unsafe {
        let fd = wasmedge_wasi_socket::TcpStream::connect(addr)?.into_raw_fd();
        TcpStream::from_std(std::net::TcpStream::from_raw_fd(fd))?
    };

    let domain = ServerName::try_from(host.to_string()).unwrap();
    let stream = connector.connect(domain, stream).await.unwrap();

    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let authority = url.authority().unwrap().clone();

    let req = Request::builder()
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())?;

    let mut res = sender.send_request(req).await?;

    println!("Response: {}", res.status());
    println!("Headers: {:#?}\n", res.headers());

    let mut resp_data = Vec::new();
    while let Some(next) = res.frame().await {
        let frame = next?;
        if let Some(chunk) = frame.data_ref() {
            resp_data.extend_from_slice(&chunk);
        }
    }

    println!("{}", String::from_utf8_lossy(&resp_data));

    Ok(())
}
