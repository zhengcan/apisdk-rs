use http::{header::CONTENT_TYPE, Extensions};
use reqwest::{Request, Response};
use reqwest_tracing::{
    default_on_request_end, default_span_name, reqwest_otel_span, ReqwestOtelSpanBackend,
};
use tracing::Span;

use super::{Logger, RequestPayload};

pub struct WithLogger {}

impl ReqwestOtelSpanBackend for WithLogger {
    fn on_request_start(req: &Request, ext: &mut Extensions) -> Span {
        let name = default_span_name(req, ext);
        let span = reqwest_otel_span!(
            name = name,
            req,
            request_id = tracing::field::Empty,
            "api.func" = tracing::field::Empty,
            "req.type" = tracing::field::Empty,
            "req.json" = tracing::field::Empty,
            "req.xml" = tracing::field::Empty,
            "req.form" = tracing::field::Empty,
            "resp.type" = tracing::field::Empty,
            "resp.json" = tracing::field::Empty,
            "resp.xml" = tracing::field::Empty,
            "resp.text" = tracing::field::Empty,
        );
        if let Some(logger) = ext.get_mut::<Logger>() {
            logger.set_span(span.clone());
            span.record("api.func", &logger.log_target);
            span.record("reques_id", &logger.request_id);
            if let Some(payload) = logger.payload.as_ref() {
                match payload {
                    RequestPayload::Json(json) => {
                        span.record("req.type", "json");
                        span.record(
                            "req.json",
                            serde_json::to_string(json).unwrap_or_else(|e| e.to_string()),
                        );
                    }
                    RequestPayload::Xml(xml) => {
                        span.record("req.type", "xml");
                        span.record("req.xml", xml);
                    }
                    RequestPayload::Form(meta) => {
                        span.record("req.type", "multipart");
                        span.record(
                            "req.form",
                            serde_json::to_string(meta).unwrap_or_else(|e| e.to_string()),
                        );
                    }
                    RequestPayload::Multipart(meta) => {
                        span.record("req.type", "form");
                        span.record(
                            "req.form",
                            serde_json::to_string(meta).unwrap_or_else(|e| e.to_string()),
                        );
                    }
                }
            }
        }
        span
    }

    fn on_request_end(
        span: &Span,
        outcome: &Result<Response, reqwest_middleware::Error>,
        _: &mut Extensions,
    ) {
        if let Ok(res) = outcome {
            if let Some(content_type) = res.headers().get(CONTENT_TYPE) {
                if let Ok(content_type) = content_type.to_str() {
                    span.record("resp.type", content_type);
                }
            }
        }
        default_on_request_end(span, outcome);
    }
}
