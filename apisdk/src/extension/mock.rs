use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Request;
use reqwest_middleware::{RequestBuilder, RequestInitialiser};
use serde_json::Value;

/// Reply json value to request. It should be used with MockServer.
#[async_trait]
pub trait JsonResponder: 'static + Send + Sync {
    /// Handle the request
    /// - req: HTTP request
    async fn handle(&self, req: Request) -> anyhow::Result<Value>;
}

/// Implement JsonResponder for function / closure
#[async_trait]
impl<F> JsonResponder for F
where
    F: 'static + Send + Sync,
    F: Fn(Request) -> anyhow::Result<Value>,
{
    async fn handle(&self, req: Request) -> anyhow::Result<Value> {
        self(req)
    }
}

/// This middleware is used to mock the response
///
/// # Examples
///
/// ### mock single request
///
/// ```
/// let req = client.get("/api/path").await?;
/// let req = req.with_extension(MockServer::new(|r| {
///     // return a fake response by using serde_json::Value
///     Ok(json!({
///         "key": "value"
///     }))
/// }));
/// let res = send!(req).await
/// ```
///
/// ### mock all requests
///
/// ```
/// let client = XxxApi::builder().with_initialiser(MockServer::new(|r| {
///     // return a fake response by using serde_json::Value
///     Ok(json!({
///         "key": "value"
///     }))
/// })).build();
/// ```
#[derive(Clone)]
pub struct MockServer {
    /// Internal responder
    inner: Arc<dyn JsonResponder>,
}

impl MockServer {
    /// Create a new instance
    pub fn new(reply: impl JsonResponder) -> Self {
        Self {
            inner: Arc::new(reply),
        }
    }
}

#[async_trait]
impl JsonResponder for MockServer {
    async fn handle(&self, req: Request) -> anyhow::Result<Value> {
        // Delegate to internal responder
        self.inner.handle(req).await
    }
}

/// Mock all requests
#[async_trait]
impl RequestInitialiser for MockServer {
    fn init(&self, req: RequestBuilder) -> RequestBuilder {
        req.with_extension(self.clone())
    }
}
