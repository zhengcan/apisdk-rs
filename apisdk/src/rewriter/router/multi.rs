use std::sync::atomic::AtomicUsize;

use async_trait::async_trait;
use url::Url;

use crate::{ApiEndpoint, ApiRouter, DefaultApiEndpoint, RouteError, UrlRewrite};

#[derive(Debug)]
enum Strategy {
    RoundRobin(AtomicUsize),
    Random,
}

#[derive(Debug)]
pub struct MultiApiRouter {
    strategy: Strategy,
    endpoints: Vec<DefaultApiEndpoint>,
}

impl MultiApiRouter {
    pub fn new_round_robin(endpoints: &[DefaultApiEndpoint]) -> Self {
        Self {
            strategy: Strategy::RoundRobin(AtomicUsize::new(0)),
            endpoints: endpoints.to_vec(),
        }
    }

    pub fn new_random(endpoints: &[DefaultApiEndpoint]) -> Self {
        Self {
            strategy: Strategy::Random,
            endpoints: endpoints.to_vec(),
        }
    }
}

#[async_trait]
impl UrlRewrite for MultiApiRouter {
    async fn rewrite(&self, url: Url) -> Url {
        url
    }
}

#[async_trait]
impl ApiRouter for MultiApiRouter {
    async fn next_endpoint(&self) -> Result<Box<dyn ApiEndpoint>, RouteError> {
        let index = match &self.strategy {
            Strategy::RoundRobin(current) => {
                current.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            }
            Strategy::Random => rand::random(),
        };
        let endpoint = self
            .endpoints
            .get(index % self.endpoints.len())
            .expect("Impossible");
        Ok(Box::new(endpoint.clone()))
    }
}
