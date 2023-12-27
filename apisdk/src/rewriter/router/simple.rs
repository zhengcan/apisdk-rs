use async_trait::async_trait;
use url::Url;

use crate::{ApiEndpoint, ApiRouter, DefaultApiEndpoint, RouteError, UrlRewrite};

#[derive(Debug)]
pub struct SimpleApiRouter {
    endpoint: DefaultApiEndpoint,
}

impl SimpleApiRouter {
    pub fn new(endpoint: impl Into<DefaultApiEndpoint>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}

#[async_trait]
impl UrlRewrite for SimpleApiRouter {
    async fn rewrite(&self, url: Url) -> Url {
        url
    }
}

#[async_trait]
impl ApiRouter for SimpleApiRouter {
    async fn next_endpoint(&self) -> Result<Box<dyn ApiEndpoint>, RouteError> {
        Ok(Box::new(self.endpoint.clone()))
    }
}
