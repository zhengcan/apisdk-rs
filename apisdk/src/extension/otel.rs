use async_trait::async_trait;
use http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{Error, Middleware, Next};

use crate::otel::*;

pub struct OtelMiddleware {
    pub name: String,
}

#[async_trait]
impl Middleware for OtelMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, Error> {
        get_active_span(|span| {
            span.add_event(
                self.name.clone(),
                vec![KeyValue::new("otel-middleware", self.name.clone())],
            );
        });
        next.run(req, extensions).await
    }
}
