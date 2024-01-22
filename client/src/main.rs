#![deny(warnings)]
#![warn(rust_2018_idioms)]

// use tokio::io::{self, AsyncWriteExt as _};

use std::{
    os::fd::{FromRawFd, IntoRawFd},
    pin::Pin,
    task::{Context, Poll},
};

use http_body_util::{BodyExt, Empty};
use hyper::{body::Bytes, Request};

use tokio::net::TcpStream;

// A simple type alias so as to DRY.
type MainResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> MainResult<()> {
    pretty_env_logger::init();

    let url_str = "http://eu.httpbin.org/get?msg=Hello";
    println!("\nGET as byte stream: {}", url_str);
    let url = url_str.parse::<hyper::Uri>().unwrap();
    if url.scheme_str() != Some("http") {
        println!("This example only works with 'http' URLs.");
        return Ok(());
    }
    fetch_url(url).await?;
    // tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let url_str = "http://eu.httpbin.org/get?msg=WasmEdge";
    println!("\nGET and get result as string: {}", url_str);
    let url = url_str.parse::<hyper::Uri>().unwrap();
    fetch_url(url).await?;
    // tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let url_str = "http://eu.httpbin.org/post";
    let post_body_str = "hello wasmedge";
    println!("\nPOST and get result as string: {}", url_str);
    println!("with a POST body: {}", post_body_str);
    let url = url_str.parse::<hyper::Uri>().unwrap();
    post_url_return_str(url, post_body_str.as_bytes()).await
}

use pin_project::pin_project;

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

async fn fetch_url(url: hyper::Uri) -> MainResult<()> {
    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(80);
    let addr = format!("{}:{}", host, port);
    let stream = unsafe {
        let fd = wasmedge_wasi_socket::TcpStream::connect(addr)?.into_raw_fd();
        TcpStream::from_std(std::net::TcpStream::from_raw_fd(fd))?
    };

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

async fn post_url_return_str(url: hyper::Uri, post_body: &'static [u8]) -> MainResult<()> {
    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(80);
    let addr = format!("{}:{}", host, port);
    let stream = unsafe {
        let fd = wasmedge_wasi_socket::TcpStream::connect(addr)?.into_raw_fd();
        TcpStream::from_std(std::net::TcpStream::from_raw_fd(fd))?
    };

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
        .method("POST")
        .header(hyper::header::HOST, authority.as_str())
        .body(http_body_util::Full::new(post_body))?;

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
