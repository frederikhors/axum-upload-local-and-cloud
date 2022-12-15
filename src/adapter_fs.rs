use crate::{adapter_trait::AdapterTrait, custom_error::Result};
use std::{path::Path, pin::Pin};
use tokio::{fs::File, io::AsyncRead};

pub struct Client {
    directory: String,
}

impl Client {
    pub fn new(directory: &str) -> Self {
        Self {
            directory: directory.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl AdapterTrait for Client {
    async fn put_file<'a>(
        &'a self,
        filename: &'a str,
        mut reader: Pin<Box<dyn AsyncRead + Send + Sync + 'a>>,
    ) -> Result<()> {
        let path = Path::new(&self.directory).join(filename);

        let mut file = File::create(path).await?;

        tokio::io::copy(&mut reader, &mut file).await?;

        Ok(())
    }

    async fn get_file(&self, filename: &str) -> Result<Pin<Box<dyn AsyncRead + Send + Sync>>> {
        let path = Path::new(&self.directory).join(filename);

        Ok(Box::pin(tokio::fs::File::open(path).await?))
    }
}
