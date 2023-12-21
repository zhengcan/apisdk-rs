use async_trait::async_trait;
use hyper::header::{HeaderValue, HOST};
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use task_local_extensions::Extensions;

pub(crate) struct RewriteHost {
    host: String,
}

impl RewriteHost {
    pub fn new(host: impl ToString) -> Self {
        Self {
            host: host.to_string(),
        }
    }
}

pub(crate) struct RewriteHostMiddleware;

#[async_trait]
impl Middleware for RewriteHostMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, reqwest_middleware::Error> {
        let mut req = req;

        // Rewrite host
        if let Some(rewriter_host) = extensions.get::<RewriteHost>() {
            let headers = req.headers_mut();
            headers.insert(
                HOST,
                HeaderValue::from_str(rewriter_host.host.as_str()).unwrap(),
            );
        }

        next.run(req, extensions).await
    }
}
