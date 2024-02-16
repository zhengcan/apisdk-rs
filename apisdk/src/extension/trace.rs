use async_trait::async_trait;
use reqwest::{header::HeaderValue, Request, Response};
use reqwest_middleware::{Middleware, Next, RequestBuilder};
use task_local_extensions::Extensions;

use crate::MiddlewareError;

/// Generate a new id for `X-Request-ID` or `X-Trace-ID`
#[cfg(not(feature = "uuid"))]
fn generate_id() -> String {
    nanoid::nanoid!()
}

/// Generate a new id for `X-Request-ID` or `X-Trace-ID`
#[cfg(feature = "uuid")]
fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// This extension will set the `X-Request-ID` header
///
/// # Example
///
/// ```
/// let req = client.get("/path").await?;
/// let req = req.with_extension(RequestId::new("new-request-id"));
/// ```
#[derive(Debug, Clone)]
pub struct RequestId {
    pub request_id: String,
}

impl Default for RequestId {
    fn default() -> Self {
        Self {
            request_id: generate_id(),
        }
    }
}

impl RequestId {
    /// Create a new RequestId
    pub fn new(request_id: impl ToString) -> Self {
        Self {
            request_id: request_id.to_string(),
        }
    }
}

/// This extension will set the `X-Trace-ID` and/or `X-Span-ID` header
///
/// # Example
///
/// ```
/// let req = client.get("/path").await?;
/// let req = req.with_extension(TraceId::new("new-trace-id", None));
/// ```
#[derive(Debug, Clone)]
pub struct TraceId {
    pub trace_id: String,
    pub span_id: Option<String>,
}

impl Default for TraceId {
    fn default() -> Self {
        Self {
            trace_id: generate_id(),
            span_id: None,
        }
    }
}

impl TraceId {
    /// Create a new TraceId
    pub fn new(trace_id: impl ToString, span_id: Option<impl ToString>) -> Self {
        Self {
            trace_id: trace_id.to_string(),
            span_id: span_id.map(|id| id.to_string()),
        }
    }
}

/// This struct is used to inject RequestId and/or TraceId to request
#[derive(Default)]
pub(crate) struct RequestTraceIdMiddleware;

impl RequestTraceIdMiddleware {
    /// This function will be invoked at the very beginning of send()
    pub(crate) fn inject_extension(req: RequestBuilder) -> RequestBuilder {
        let mut req = req;

        let (request_id, trace_id) = (
            req.extensions()
                .get::<RequestId>()
                .map(|id| id.request_id.clone()),
            req.extensions()
                .get::<TraceId>()
                .map(|id| (Some(id.trace_id.clone())))
                .unwrap_or(None),
        );

        match (request_id, trace_id) {
            (Some(id), None) => req.with_extension(TraceId::new(id, None::<&str>)),
            (None, Some(id)) => req.with_extension(RequestId::new(id)),
            (None, None) => {
                let id = generate_id();
                req.with_extension(RequestId::new(&id))
                    .with_extension(TraceId::new(id, None::<&str>))
            }
            _ => req,
        }
    }

    /// This function will be invoked at the end of send()
    pub(crate) fn inject_header(req: Request, extensions: &Extensions) -> Request {
        let mut req = req;
        let headers = req.headers_mut();

        // X-Request-ID
        if !headers.contains_key("X-Request-ID") {
            let request_id = extensions
                .get::<RequestId>()
                .map(|id| id.request_id.clone())
                .unwrap_or_else(generate_id);
            headers.insert("X-Request-ID", HeaderValue::from_str(&request_id).unwrap());
        }

        // X-Trace-ID & X-Span-ID
        if !headers.contains_key("X-Trace-ID") {
            let (trace_id, span_id) = match extensions.get::<TraceId>() {
                Some(id) => (id.trace_id.clone(), id.span_id.clone()),
                None => (generate_id(), None),
            };
            headers.insert("X-Trace-ID", HeaderValue::from_str(&trace_id).unwrap());
            if let Some(span_id) = span_id {
                headers.insert("X-Span-ID", HeaderValue::from_str(&span_id).unwrap());
            }
        }

        req
    }
}

/// Using `Middleware`, the injector will set request headers
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl Middleware for RequestTraceIdMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, MiddlewareError> {
        let req = Self::inject_header(req, extensions);
        next.run(req, extensions).await
    }
}
