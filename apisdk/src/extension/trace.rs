use async_trait::async_trait;
use reqwest::{header::HeaderValue, Request, Response};
use reqwest_middleware::{Middleware, Next, RequestBuilder};
use task_local_extensions::Extensions;

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

struct DefaultId(String);

impl Default for DefaultId {
    fn default() -> Self {
        Self(generate_id())
    }
}

/// This struct is used to inject RequestId and/or TraceId to request
#[derive(Default)]
pub(crate) struct RequestTraceIdInjector;

impl RequestTraceIdInjector {
    /// This function will be invoked at the very beginning of send()
    pub(crate) fn inject_to_builder(req: RequestBuilder) -> RequestBuilder {
        let mut req = req;

        // X-Request-ID
        let request_id = if let Some(request_id) = req
            .extensions()
            .get::<RequestId>()
            .map(|id| id.request_id.clone())
        {
            req = req.header("X-Request-ID", &request_id);
            Some(request_id)
        } else {
            None
        };

        // X-Trace-ID & X-Span-ID
        let trace_id = if let Some((trace_id, span_id)) = req
            .extensions()
            .get::<TraceId>()
            .map(|id| (id.trace_id.clone(), id.span_id.clone()))
        {
            req = req.header("X-Trace-ID", &trace_id);
            if let Some(span_id) = span_id {
                req = req.header("X-Span-ID", span_id);
            }
            Some(trace_id)
        } else {
            None
        };

        match (request_id, trace_id) {
            (Some(id), None) | (None, Some(id)) => req = req.with_extension(DefaultId(id)),
            (None, None) => req = req.with_extension(DefaultId::default()),
            _ => {}
        };

        req
    }

    /// This function will be invoked at the end of send()
    pub(crate) fn inject_to_request(req: Request, extensions: &Extensions) -> Request {
        let mut req = req;
        let headers = req.headers_mut();
        let default_id = extensions
            .get::<DefaultId>()
            .map(|id| id.0.clone())
            .unwrap_or_else(generate_id);

        // X-Request-ID
        if !headers.contains_key("X-Request-ID") {
            let request_id = extensions
                .get::<RequestId>()
                .map(|id| id.request_id.clone())
                .unwrap_or_else(|| default_id.clone());
            headers.insert("X-Request-ID", HeaderValue::from_str(&request_id).unwrap());
        }

        // X-Trace-ID & X-Span-ID
        if !headers.contains_key("X-Trace-ID") {
            let (trace_id, span_id) = match extensions.get::<TraceId>() {
                Some(id) => (id.trace_id.clone(), id.span_id.clone()),
                None => (default_id, None),
            };
            headers.insert("X-Trace-ID", HeaderValue::from_str(&trace_id).unwrap());
            if let Some(span_id) = span_id {
                headers.insert("X-Span-ID", HeaderValue::from_str(&span_id).unwrap());
            }
        }

        req
    }
}

// /// The injector will inject new ids as default.
// /// Developers can overwrite them.
// #[async_trait]
// impl RequestInitialiser for RequestTraceIdInjector {
//     fn init(&self, req: RequestBuilder) -> RequestBuilder {
//         let new_id = generate_id();
//         req.with_extension(RequestId::new(&new_id))
//             .with_extension(TraceId::new(&new_id, None::<&str>))
//     }
// }

/// Using `Middleware`, the injector will set request headers
#[async_trait]
impl Middleware for RequestTraceIdInjector {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, reqwest_middleware::Error> {
        let req = Self::inject_to_request(req, extensions);
        next.run(req, extensions).await
    }
}
