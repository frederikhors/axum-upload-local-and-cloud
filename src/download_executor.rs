use std::{pin::Pin, sync::Arc};
use tokio::io::AsyncRead;

use crate::{adapter_trait::AdapterTrait, custom_error::Result};

pub struct Executor {
    client: Arc<dyn AdapterTrait>,
}

impl Executor {
    pub fn new(client: Arc<dyn AdapterTrait>) -> Self {
        Self { client }
    }

    pub async fn execute(
        &self,
        filename: &str,
    ) -> Result<Option<Pin<Box<dyn AsyncRead + Send + Sync>>>> {
        let file = self.client.get_file(filename).await?;

        Ok(Some(file))
    }
}
