use crate::custom_error::Result;
use std::pin::Pin;
use tokio::io::AsyncRead;

#[async_trait::async_trait]
pub trait AdapterTrait: Send + Sync {
    async fn put_file<'a>(
        &'a self,
        filename: &'a str,
        reader: Pin<Box<dyn AsyncRead + Send + Sync + 'a>>,
    ) -> Result<()>;

    async fn get_file(&self, filename: &str) -> Result<Pin<Box<dyn AsyncRead + Send + Sync>>>;
}
