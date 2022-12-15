use crate::{adapter_trait::AdapterTrait, custom_error::Result};
use std::{pin::Pin, sync::Arc};
use tokio::io::AsyncRead;

pub struct Executor {
    client: Arc<dyn AdapterTrait>,
}

impl Executor {
    pub fn new(client: Arc<dyn AdapterTrait>) -> Self {
        Self { client }
    }

    pub async fn execute<'a>(
        &'a self,
        filename: &'a str,
        reader: Pin<Box<(dyn AsyncRead + Send + Sync + 'a)>>,
    ) -> Result<()> {
        self.client.put_file(&filename, reader).await?;

        Ok(())
    }
}
