use adapter_trait::AdapterTrait;
use axum::{
    body::{boxed, StreamBody},
    extract::{Multipart, Query},
    http::StatusCode,
    response::Response,
    routing::{on, MethodFilter},
    Extension, Router,
};
use custom_error::Result;
use futures::TryStreamExt;
use std::{io, net::SocketAddr, sync::Arc};
use tokio_util::io::{ReaderStream, StreamReader};

pub mod adapter_fs;
pub mod adapter_s3;
pub mod adapter_trait;
pub mod custom_error;
pub mod download_executor;
pub mod upload_executor;

const USE_CLOUD: bool = true;

pub struct AppState {
    pub upload_executor: upload_executor::Executor,
    pub download_executor: download_executor::Executor,
}

#[tokio::main]
async fn main() {
    let (upload_executor, download_executor) = init_clients();

    let app_state = Arc::new(AppState {
        upload_executor,
        download_executor,
    });

    let router = Router::new()
        .route("/upload", on(MethodFilter::POST, upload))
        .route("/download/*key", on(MethodFilter::GET, download))
        .layer(Extension(app_state));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

pub async fn upload(
    app_state: Extension<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Response, StatusCode> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let filename = if let Some(filename) = field.file_name() {
            filename.to_string()
        } else {
            continue;
        };

        let body_with_io_error = field.map_err(|err| io::Error::new(io::ErrorKind::Other, err));

        let body_reader = StreamReader::new(body_with_io_error);

        app_state
            .upload_executor
            .execute(&filename, Box::pin(body_reader))
            .await
            .unwrap();

        return Ok(Response::builder()
            .status(StatusCode::CREATED)
            .body(boxed("OK".to_string()))
            .unwrap());
    }

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn download(
    app_state: Extension<Arc<AppState>>,
    Query(params): Query<Vec<(String, String)>>,
) -> Result<Response, StatusCode> {
    let filename = params[0].1.to_string();

    let stream = app_state
        .download_executor
        .execute(&filename)
        .await
        .unwrap();

    Ok(Response::builder()
        .body(boxed(StreamBody::new(ReaderStream::new(stream.unwrap()))))
        .unwrap())
}

fn init_clients() -> (upload_executor::Executor, download_executor::Executor) {
    let client_fs = if USE_CLOUD {
        None
    } else {
        Some(Arc::new(adapter_fs::Client::new(".")))
    };

    let client_cloud = if USE_CLOUD {
        Some(Arc::new(
            adapter_s3::Client::new(
                "https://s3.region.aws.com",
                "region",
                "bucket_name",
                "KEY_ID",
                "KEY_SECRET",
            )
            .unwrap(),
        ))
    } else {
        None
    };

    let client = match USE_CLOUD {
        true => client_cloud
            .as_ref()
            .map(|o| o.clone() as Arc<dyn AdapterTrait>)
            .unwrap(),
        false => client_fs
            .as_ref()
            .map(|o| o.clone() as Arc<dyn AdapterTrait>)
            .unwrap(),
    };

    let upload_executor = upload_executor::Executor::new(client.clone());

    let download_executor = download_executor::Executor::new(client);

    (upload_executor, download_executor)
}
