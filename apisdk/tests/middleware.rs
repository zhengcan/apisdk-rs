use std::time::Instant;

use apisdk::{send, ApiResult, CodeDataMessage};
use http::Extensions;
use reqwest::{Request, Response};
use reqwest_tracing::{
    default_on_request_end, reqwest_otel_span, ReqwestOtelSpanBackend, TracingMiddleware,
};
use tracing::Span;

use crate::common::{init_logger, start_server, Payload, TheApi};

mod common;

impl TheApi {
    async fn touch(&self) -> ApiResult<Payload> {
        let req = self.get("/path/json").await?;
        send!(req, CodeDataMessage).await
    }
}

pub struct TimeTrace;

impl ReqwestOtelSpanBackend for TimeTrace {
    fn on_request_start(req: &Request, extension: &mut Extensions) -> Span {
        extension.insert(Instant::now());
        reqwest_otel_span!(
            name = "example-request",
            req,
            time_elapsed = tracing::field::Empty
        )
    }

    fn on_request_end(
        span: &Span,
        outcome: &Result<Response, reqwest_middleware::Error>,
        extension: &mut Extensions,
    ) {
        let time_elapsed = extension.get::<Instant>().unwrap().elapsed().as_millis() as i64;
        default_on_request_end(span, outcome);
        span.record("time_elapsed", &time_elapsed);
    }
}

#[tokio::test]
async fn test_opentelemetry() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder()
        .with_middleware(TracingMiddleware::<TimeTrace>::new())
        .build();

    let res = api.touch().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}
