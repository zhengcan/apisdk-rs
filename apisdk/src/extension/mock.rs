use std::{any::type_name, sync::Arc};

use async_trait::async_trait;
use reqwest::Request;
use reqwest_middleware::{RequestBuilder, RequestInitialiser};

use crate::ResponseBody;

/// Reply a response to request. It should be used with MockServer.
#[async_trait]
pub trait Responder: 'static + Send + Sync {
    /// Get type_name, used in Debug
    fn type_name(&self) -> &str {
        type_name::<Self>()
    }

    /// Handle the request
    /// - req: HTTP request
    async fn handle(&self, req: Request) -> anyhow::Result<ResponseBody>;
}

/// Implement Responder for function / closure
#[async_trait]
impl<F> Responder for F
where
    F: 'static + Send + Sync,
    F: Fn(Request) -> anyhow::Result<ResponseBody>,
{
    async fn handle(&self, req: Request) -> anyhow::Result<ResponseBody> {
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
    inner: Arc<dyn Responder>,
}

impl MockServer {
    /// Create a new instance
    pub fn new(reply: impl Responder) -> Self {
        Self {
            inner: Arc::new(reply),
        }
    }
}

#[async_trait]
impl Responder for MockServer {
    fn type_name(&self) -> &str {
        self.inner.type_name()
    }

    async fn handle(&self, req: Request) -> anyhow::Result<ResponseBody> {
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
