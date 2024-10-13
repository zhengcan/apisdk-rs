#[cfg(feature = "opentelemetry_0_24")]
mod opentelemetry_0_24 {
    pub use opentelemetry_0_24_pkg::trace::*;
    pub use opentelemetry_0_24_pkg::KeyValue;
}

#[cfg(feature = "opentelemetry_0_24")]
pub use opentelemetry_0_24::*;

#[cfg(feature = "opentelemetry_0_25")]
mod opentelemetry_0_25 {
    pub use opentelemetry_0_25_pkg::trace::*;
    pub use opentelemetry_0_25_pkg::KeyValue;
}

#[cfg(feature = "opentelemetry_0_25")]
pub use opentelemetry_0_25::*;

#[cfg(feature = "opentelemetry_0_26")]
mod opentelemetry_0_26 {
    pub use opentelemetry_0_26_pkg::trace::*;
    pub use opentelemetry_0_26_pkg::KeyValue;
}

#[cfg(feature = "opentelemetry_0_26")]
pub use opentelemetry_0_26::*;

use http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{Error, Next};

pub struct OtelMiddleware {
    pub name: String,
}

#[async_trait::async_trait]
impl crate::Middleware for OtelMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, Error> {
        use crate::otel::*;
        get_active_span(|span| {
            span.add_event(
                self.name.clone(),
                vec![KeyValue::new("otel-middleware", self.name.clone())],
            );
        });
        next.run(req, extensions).await
    }
}
