use async_trait::async_trait;

use crate::{ApiEndpoint, ApiRouter, DefaultApiEndpoint, RouteError};

#[derive(Debug)]
pub struct SingleApiRouter {
    endpoint: DefaultApiEndpoint,
}

impl SingleApiRouter {
    pub fn new(endpoint: impl Into<DefaultApiEndpoint>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}

#[async_trait]
impl ApiRouter for SingleApiRouter {
    async fn next_endpoint(&self) -> Result<Box<dyn ApiEndpoint>, RouteError> {
        Ok(Box::new(self.endpoint.clone()))
    }
}
