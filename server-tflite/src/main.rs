use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use image::io::Reader;
use image::DynamicImage;
use std::io::Cursor;
use std::net::SocketAddr;
use std::os::fd::{FromRawFd, IntoRawFd};
use std::pin::Pin;
use std::result::Result;
use std::task::{Context, Poll};
use tokio::net::TcpListener;
use wasi_nn::{ExecutionTarget, GraphBuilder, GraphEncoding, TensorType};

/// This is our service handler. It receives a Request, routes on its
/// path, and returns a Future of a Response.
async fn classify(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, anyhow::Error> {
    let model_data: &[u8] =
        include_bytes!("models/mobilenet_v1_1.0_224/mobilenet_v1_1.0_224_quant.tflite");
    let labels = include_str!("models/mobilenet_v1_1.0_224/labels_mobilenet_quant_v1_224.txt");
    let graph = GraphBuilder::new(GraphEncoding::TensorflowLite, ExecutionTarget::CPU)
        .build_from_bytes(&[model_data])?;
    let mut ctx = graph.init_execution_context()?;
    /*
    let graph = unsafe {
        wasi_nn::load(
            &[model_data],
            4, // encoding for tflite: wasi_nn::GRAPH_ENCODING_TENSORFLOWLITE
            wasi_nn::EXECUTION_TARGET_CPU,
        )
        .unwrap()
    };
    let context = unsafe { wasi_nn::init_execution_context(graph).unwrap() };
    */

    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => Ok(Response::new(full(
            "Try POSTing data to /classify such as: `curl http://localhost:3000/classify -X POST --data-binary '@grace_hopper.jpg'`",
        ))),

        (&Method::POST, "/classify") => {
            let buf = req.collect().await?.to_bytes();

            let tensor_data = image_to_tensor(&buf, 224, 224);
          ctx.set_input(0, TensorType::U8, &[1, 224, 224, 3], &tensor_data)?;
            /*
            let tensor = wasi_nn::Tensor {
                dimensions: &[1, 224, 224, 3],
                r#type: wasi_nn::TENSOR_TYPE_U8,
                data: &tensor_data,
            };
            unsafe {
                wasi_nn::set_input(context, 0, tensor).unwrap();
            }
            */
            // Execute the inference.
            ctx.compute()?;
            /*
            unsafe {
                wasi_nn::compute(context).unwrap();
            }
            */
            // Retrieve the output.
            let mut output_buffer = vec![0u8; labels.lines().count()];
            _ = ctx.get_output(0, &mut output_buffer)?;
            /*
            unsafe {
                wasi_nn::get_output(
                    context,
                    0,
                    &mut output_buffer[..] as *mut [u8] as *mut u8,
                    output_buffer.len().try_into().unwrap(),
                )
                .unwrap();
            }
            */
            // Sort the result with the highest probability result first
            let results = sort_results(&output_buffer);
            /*
            for r in results.iter() {
                println!("results: {} {}", r.0, r.1);
            }
            */
            // The first result's first element points to the labels position
            let class_name = labels.lines().nth(results[0].0).unwrap_or("Unknown");
            println!("result: {} {}", class_name, results[0].1);

            Ok(Response::new(full(format!("{} is detected with {}/255 confidence", class_name, results[0].1))))
        }

        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
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

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = unsafe {
        let fd = wasmedge_wasi_socket::TcpListener::bind(addr, true)?.into_raw_fd();
        TcpListener::from_std(std::net::TcpListener::from_raw_fd(fd))?
    };

    loop {
        let (stream, _) = listener.accept().await?;
        println!("accept");
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(classify))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }

    /*
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);
    loop {
        let (stream, _) = listener.accept().await?;

        tokio::task::spawn(async move {
            if let Err(err) = Http::new().serve_connection(stream, service_fn(classify)).await {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
    */
}

// Sort the buffer of probabilities. The graph places the match probability for each class at the
// index for that class (e.g. the probability of class 42 is placed at buffer[42]). Here we convert
// to a wrapping InferenceResult and sort the results.
fn sort_results(buffer: &[u8]) -> Vec<InferenceResult> {
    let mut results: Vec<InferenceResult> = buffer
        .iter()
        .enumerate()
        .map(|(c, p)| InferenceResult(c, *p))
        .collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    results
}

// Take the image data, resize it to height x width, and then convert
// the pixel precision to FP32. The resulting BGR pixel vector is then returned.
fn image_to_tensor(raw_data: &[u8], height: u32, width: u32) -> Vec<u8> {
    let reader = Reader::new(Cursor::new(raw_data))
        .with_guessed_format()
        .expect("Cursor io never fails");
    let pixels = reader.decode().unwrap();
    let dyn_img: DynamicImage = pixels.resize_exact(width, height, image::imageops::Triangle);
    let bgr_img = dyn_img.to_rgb8();
    // Get an array of the pixel values
    let raw_u8_arr: &[u8] = &bgr_img.as_raw()[..];
    return raw_u8_arr.to_vec();
}

// A wrapper for class ID and match probabilities.
#[derive(Debug, PartialEq)]
struct InferenceResult(usize, u8);
