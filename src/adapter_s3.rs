use crate::{adapter_trait::AdapterTrait, custom_error::Result};
use futures::TryStreamExt;
use rusty_s3::{Bucket, Credentials, S3Action, UrlStyle};
use std::{pin::Pin, time::Duration};
use tokio::io::AsyncRead;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tokio_util::io::ReaderStream;

pub struct Client {
    bucket: Bucket,
    credentials: Credentials,
}

impl Client {
    pub fn new(
        endpoint: &str,
        region: &str,
        bucket_name: &str,
        key_id: &str,
        key_secret: &str,
    ) -> Result<Self> {
        let bucket = Bucket::new(
            endpoint.to_string().parse().unwrap(),
            UrlStyle::VirtualHost,
            bucket_name.to_string(),
            region.to_string(),
        )
        .expect("Url has a valid scheme and host");

        let credentials = Credentials::new(key_id, key_secret);

        Ok(Self {
            bucket,
            credentials,
        })
    }
}

#[async_trait::async_trait]
impl AdapterTrait for Client {
    async fn put_file<'a>(
        &'a self,
        filename: &'a str,
        reader: Pin<Box<dyn AsyncRead + Send + Sync + 'a>>,
    ) -> Result<()> {
        let presigned_url_duration = Duration::from_secs(60 * 60);

        let action = self.bucket.put_object(Some(&self.credentials), filename);

        let client = reqwest::Client::new();

        client
            .post(action.sign(presigned_url_duration))
            // .body(reader)
            // .body(reqwest::Body::wrap_stream(reader))
            .body(reqwest::Body::wrap_stream(ReaderStream::new(reader)))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn get_file(&self, filename: &str) -> Result<Pin<Box<dyn AsyncRead + Send + Sync>>> {
        let presigned_url_duration = Duration::from_secs(60 * 60);

        let action = self.bucket.get_object(Some(&self.credentials), filename);

        let resp = reqwest::get(action.sign(presigned_url_duration))
            .await?
            .error_for_status()?
            .bytes_stream()
            .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
            .into_async_read()
            .compat();

        Ok(Box::pin(resp))
    }
}
