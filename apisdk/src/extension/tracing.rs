use http::Extensions;
use reqwest::{Request, Response};
use reqwest_tracing::{
    default_on_request_end, default_span_name, reqwest_otel_span, ReqwestOtelSpanBackend,
};
use tracing::Span;

use super::Logger;

pub struct WithLogger {}

impl ReqwestOtelSpanBackend for WithLogger {
    fn on_request_start(req: &Request, ext: &mut Extensions) -> Span {
        let logger = ext.get::<Logger>();
        let name = default_span_name(req, ext);
        reqwest_otel_span!(
            name = name,
            req,
            "req.type" = tracing::field::Empty,
            "req.json" = tracing::field::Empty,
            "req.xml" = tracing::field::Empty,
            "req.multipart" = tracing::field::Empty,
            "req.form" = tracing::field::Empty,
            "resp.type" = tracing::field::Empty,
            "resp.json" = tracing::field::Empty,
            "resp.xml" = tracing::field::Empty,
            "resp.text" = tracing::field::Empty,
            "req.logger" = format!("{:?}", logger),
            "resp.logger" = tracing::field::Empty,
        )
    }

    fn on_request_end(
        span: &Span,
        outcome: &Result<Response, reqwest_middleware::Error>,
        ext: &mut Extensions,
    ) {
        let logger = ext.get::<Logger>();
        default_on_request_end(span, outcome);
        span.record("resp.logger", format!("{:?}", logger));
    }
}
