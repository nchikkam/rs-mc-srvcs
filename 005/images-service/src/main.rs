use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use futures::stream::TryStreamExt;
use lazy_static::lazy_static;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use regex::Regex;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio_util::codec::{BytesCodec, FramedRead};

use hyper::{Body, Method, Request, Response, StatusCode, Server};
use hyper::service::{make_service_fn, service_fn};

static INDEX: &[u8] = b"Images Microservice";

lazy_static! {
    static ref DOWNLOAD_FILE: Regex = Regex::new("^/download/(?P<filename>\\w{20})$").unwrap();
}

async fn microservice_handler(
    req: Request<Body>,
    files: Arc<PathBuf>,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        // Root
        (&Method::GET, "/") => Ok(Response::new(INDEX.into())),

        // Upload
        (&Method::POST, "/upload") => {
            let name: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(20)
                .map(char::from)
                .collect();

            let mut filepath = files.as_ref().clone();
            filepath.push(&name);

            let mut f = match File::create(&filepath).await {
                Ok(file) => file,
                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Failed to create file"))
                        .unwrap())
                }
            };

            let mut body = req.into_body();
            while let Some(chunk) = body.try_next().await.unwrap_or(None) {
                if let Err(_) = f.write_all(&chunk).await {
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Failed to write file"))
                        .unwrap());
                }
            }

            Ok(Response::new(name.into()))
        },

        // Download
        (&Method::GET, path) if path.starts_with("/download/") => {
            if let Some(cap) = DOWNLOAD_FILE.captures(path) {
                let filename = cap.name("filename").unwrap().as_str();
                let mut filepath = files.as_ref().clone();
                filepath.push(filename);

                match File::open(&filepath).await {
                    Ok(file) => {
                        let stream = FramedRead::new(file, BytesCodec::new());
                        Ok(Response::new(Body::wrap_stream(stream)))
                    }
                    Err(_) => Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::from("File not found"))
                        .unwrap()),
                }
            } else {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("Invalid download path"))
                    .unwrap())
            }
        },

        // Fallback
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap()),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let files = Arc::new(PathBuf::from("./files"));
    fs::create_dir_all(&*files)?;

    let addr = ([127, 0, 0, 1], 8080).into();

    let make_svc = make_service_fn(move |_| {
        let files = files.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                microservice_handler(req, files.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Serving on http://{}", addr);
    server.await?;

    Ok(())
}