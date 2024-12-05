use std::collections::HashMap;

use http::StatusCode;
use reqwest::{header::CONTENT_TYPE, Response, ResponseBuilderExt};
use serde::Serialize;
use serde_json::Value;
#[cfg(feature = "tracing")]
use tracing::Instrument;

use crate::{
    get_default_log_level, ApiError, ApiResult, FormLike, IntoFilter, LogConfig, Logger, MimeType,
    MockServer, RequestBuilder, RequestId, RequestTraceIdMiddleware, Responder, ResponseBody,
};

/// This struct is used to build RequestConfig internally by macros.
#[derive(Debug, Default)]
pub struct RequestConfigurator {
    /// The target of log
    log_target: &'static str,
    /// Indicate whether to log
    log_filter: Option<log::LevelFilter>,
    /// Indicate whether to parse headers from response or not
    require_headers: bool,
}

impl RequestConfigurator {
    /// Create a new instance
    pub fn new(
        log_target: &'static str,
        log_filter: Option<impl IntoFilter>,
        require_headers: bool,
    ) -> Self {
        Self {
            log_target,
            log_filter: log_filter.and_then(|f| f.into_filter()),
            require_headers,
        }
    }

    /// Update config
    pub fn merge(self, log_target: &'static str, require_headers: bool) -> Self {
        RequestConfigurator {
            log_target,
            require_headers,
            ..self
        }
    }

    #[cfg(feature = "tracing")]
    fn get_caller(&self) -> &str {
        match self.log_target.rsplit_once("::") {
            Some((_, fn_name)) => fn_name,
            None => self.log_target,
        }
    }

    /// Build Logger
    fn build(self, req: &mut RequestBuilder) -> (Logger, bool) {
        let extensions = req.extensions();

        let log_filter = extensions
            .get::<LogConfig>()
            .map(|config| config.level)
            .or(self.log_filter)
            .unwrap_or(get_default_log_level());

        let request_id = extensions
            .get::<RequestId>()
            .map(|id| id.request_id.clone())
            .unwrap_or_default();

        (
            Logger::new(self.log_target, log_filter, request_id),
            self.require_headers,
        )
    }
}

/// Send request
/// - req: used to build request
/// - config: control the send process
pub async fn send(req: RequestBuilder, config: RequestConfigurator) -> ApiResult<ResponseBody> {
    #[cfg(feature = "tracing")]
    {
        let span = tracing::info_span!(
            "API call / send",
            otel.name = format!("[API] {}", config.get_caller()),
            "api.func" = config.log_target,
            "resp.type" = tracing::field::Empty,
            "error" = tracing::field::Empty,
            "exception" = tracing::field::Empty,
        );
        with_span(do_send(req, config), span, || {}).await
    }
    #[cfg(not(feature = "tracing"))]
    do_send(req, config).await
}
async fn do_send(mut req: RequestBuilder, config: RequestConfigurator) -> ApiResult<ResponseBody> {
    // Inject extensions
    req = RequestTraceIdMiddleware::inject_extension(req);
    let (logger, require_headers) = config.build(&mut req);
    if logger.is_enabled() {
        req = req.with_extension(logger.clone());
    }

    send_and_parse(req, logger, require_headers).await
}

/// Send request with JSON payload
/// - req: used to build request
/// - json: request payload
/// - config: control the send process
pub async fn send_json<I>(
    req: RequestBuilder,
    json: &I,
    config: RequestConfigurator,
) -> ApiResult<ResponseBody>
where
    I: Serialize + ?Sized,
{
    let req = req.json(json);

    #[cfg(feature = "tracing")]
    {
        let span = tracing::info_span!(
            "API call / send_json",
            otel.name = format!("[API] {}", config.get_caller()),
            "api.func" = config.log_target,
            "req.type" = "json",
            "resp.type" = tracing::field::Empty,
            "error" = tracing::field::Empty,
            "exception" = tracing::field::Empty,
        );
        with_span(do_send_json(req, json, config), span, || {
            tracing::info!(
                name = "request",
                json = serde_json::to_string(json).unwrap_or_default(),
                "request.json",
            );
        })
        .await
    }
    #[cfg(not(feature = "tracing"))]
    do_send_json(req, json, config).await
}

async fn do_send_json<I>(
    mut req: RequestBuilder,
    json: &I,
    config: RequestConfigurator,
) -> ApiResult<ResponseBody>
where
    I: Serialize + ?Sized,
{
    // Inject extensions
    req = RequestTraceIdMiddleware::inject_extension(req);
    let (logger, require_headers) = config.build(&mut req);
    if logger.is_enabled() {
        req = req.with_extension(
            logger
                .clone()
                .with_json(serde_json::to_value(json).unwrap_or_default()),
        );
    }

    send_and_parse(req, logger, require_headers).await
}

/// Send request with xml payload
/// - req: used to build request
/// - form: request payload
/// - config: control the send process
pub async fn send_xml<I>(
    req: RequestBuilder,
    xml: &I,
    config: RequestConfigurator,
) -> ApiResult<ResponseBody>
where
    I: Serialize + ?Sized,
{
    let xml = quick_xml::se::to_string(xml)?;
    let req = req.header(CONTENT_TYPE, MimeType::Xml).body(xml.clone());

    #[cfg(feature = "tracing")]
    {
        let span = tracing::info_span!(
            "API call / send_xml",
            otel.name = format!("[API] {}", config.get_caller()),
            "api.func" = config.log_target,
            "req.type" = "xml",
            "resp.type" = tracing::field::Empty,
            "error" = tracing::field::Empty,
            "exception" = tracing::field::Empty,
        );
        with_span(do_send_xml(req, xml.clone(), config), span, || {
            tracing::info!(name = "request", xml = xml, "request.xml",);
        })
        .await
    }
    #[cfg(not(feature = "tracing"))]
    do_send_xml(req, xml, config).await
}

async fn do_send_xml(
    mut req: RequestBuilder,
    xml: String,
    config: RequestConfigurator,
) -> ApiResult<ResponseBody> {
    // Inject extensions
    req = RequestTraceIdMiddleware::inject_extension(req);
    let (logger, require_headers) = config.build(&mut req);
    if logger.is_enabled() {
        req = req.with_extension(logger.clone().with_xml(xml));
    }

    send_and_parse(req, logger, require_headers).await
}

/// Send request with form payload
/// - req: used to build request
/// - form: request payload
/// - config: control the send process
pub async fn send_form<I>(
    mut req: RequestBuilder,
    form: I,
    config: RequestConfigurator,
) -> ApiResult<ResponseBody>
where
    I: FormLike,
{
    let is_multipart = form.is_multipart();
    let meta = form.get_meta();

    if is_multipart {
        if let Some(multipart) = form.get_multipart() {
            req = req.multipart(multipart)
        }
    } else if let Some(form) = form.get_form() {
        req = req.form(&form);
    };

    #[cfg(feature = "tracing")]
    {
        let type_name = if is_multipart { "multipart" } else { "form" };
        let span = tracing::info_span!(
            "API call / send_form",
            otel.name = format!("[API] {}", config.get_caller()),
            "api.func" = config.log_target,
            "req.type" = type_name,
            "resp.type" = tracing::field::Empty,
            "error" = tracing::field::Empty,
            "exception" = tracing::field::Empty,
        );
        with_span(
            do_send_form(req, is_multipart, meta.clone(), config),
            span,
            || {
                tracing::info!(
                    name = "request",
                    form = serde_json::to_string(&meta).unwrap_or_default(),
                    "request.{}",
                    type_name,
                );
            },
        )
        .await
    }
    #[cfg(not(feature = "tracing"))]
    do_send_form(req, is_multipart, meta, config).await
}

async fn do_send_form(
    mut req: RequestBuilder,
    is_multipart: bool,
    meta: HashMap<String, String>,
    config: RequestConfigurator,
) -> ApiResult<ResponseBody> {
    // Inject extensions
    req = RequestTraceIdMiddleware::inject_extension(req);
    let (logger, require_headers) = config.build(&mut req);
    if logger.is_enabled() {
        let logger = if is_multipart {
            logger.clone().with_multipart(meta)
        } else {
            logger.clone().with_form(meta)
        };
        req = req.with_extension(logger);
    }

    send_and_parse(req, logger, require_headers).await
}

/// Send request with multipart/data payload
/// - req: used to build request
/// - form: request payload
/// - config: control the send process
pub async fn send_multipart<I>(
    mut req: RequestBuilder,
    form: I,
    config: RequestConfigurator,
) -> ApiResult<ResponseBody>
where
    I: FormLike,
{
    let form = form.get_multipart().ok_or(ApiError::MultipartForm)?;
    let meta = form.get_meta();

    req = req.multipart(form);

    #[cfg(feature = "tracing")]
    {
        let span = tracing::info_span!(
            "API call / send_multipart",
            otel.name = format!("[API] {}", config.get_caller()),
            "api.func" = config.log_target,
            "req.type" = "multipart",
            "resp.type" = tracing::field::Empty,
            "error" = tracing::field::Empty,
            "exception" = tracing::field::Empty,
        );
        with_span(do_send_multipart(req, meta.clone(), config), span, || {
            tracing::info!(
                name = "request",
                form = serde_json::to_string(&meta).unwrap_or_default(),
                "request.multipart"
            );
        })
        .await
    }
    #[cfg(not(feature = "tracing"))]
    do_send_multipart(req, meta, config).await
}

async fn do_send_multipart(
    mut req: RequestBuilder,
    meta: HashMap<String, String>,
    config: RequestConfigurator,
) -> ApiResult<ResponseBody> {
    // Inject extensions
    req = RequestTraceIdMiddleware::inject_extension(req);
    let (logger, require_headers) = config.build(&mut req);
    if logger.is_enabled() {
        req = req.with_extension(logger.clone().with_multipart(meta));
    }

    send_and_parse(req, logger, require_headers).await
}

/// Send request, and get raw response
/// - req: used to build request
/// - config: control the send process
pub async fn send_raw(req: RequestBuilder, config: RequestConfigurator) -> ApiResult<Response> {
    #[cfg(feature = "tracing")]
    {
        let span = tracing::info_span!(
            "API call / send_raw",
            otel.name = format!("[API] {}", config.get_caller()),
            "api.func" = config.log_target,
            "req.type" = "raw",
            "resp.type" = tracing::field::Empty,
            "error" = tracing::field::Empty,
            "exception" = tracing::field::Empty,
        );
        with_span_raw(do_send_raw(req, config), span).await
    }
    #[cfg(not(feature = "tracing"))]
    do_send_raw(req, config).await
}

async fn do_send_raw(mut req: RequestBuilder, config: RequestConfigurator) -> ApiResult<Response> {
    // Inject extensions
    req = RequestTraceIdMiddleware::inject_extension(req);
    let (logger, _) = config.build(&mut req);
    if logger.is_enabled() {
        req = req.with_extension(logger.clone());
    }

    send_and_unparse(req, logger).await
}

/// Send request with a tracing span
#[cfg(feature = "tracing")]
async fn with_span<F, I>(f: F, span: tracing::Span, init: I) -> Result<ResponseBody, ApiError>
where
    F: std::future::Future<Output = Result<ResponseBody, ApiError>>,
    I: Fn(),
{
    let future = async {
        init();
        let outcome = f.await;
        match outcome.as_ref() {
            Ok(response) => match response {
                ResponseBody::Empty => {
                    span.record("resp.type", "empty");
                    tracing::info!(name = "response", "response.empty",);
                }
                ResponseBody::Json(value) => {
                    span.record("resp.type", "json");
                    tracing::info!(
                        name = "response",
                        json = serde_json::to_string(value).unwrap_or_default(),
                        "response.json",
                    );
                }
                ResponseBody::Xml(xml) => {
                    span.record("resp.type", "xml");
                    tracing::info!(name = "response", xml = xml, "response.xml",);
                }
                ResponseBody::Text(text) => {
                    span.record("resp.type", "text");
                    tracing::info!(name = "response", text = text, "response.text",);
                }
            },
            Err(e) => {
                span.record("error", true);
                span.record("exception", e.to_string());
                tracing::warn!(
                    name = "exception",
                    exception = e.to_string(),
                    "response.error",
                );
            }
        }
        outcome
    };
    future.instrument(span.clone()).await
}

/// Send request with a tracing span
#[cfg(feature = "tracing")]
async fn with_span_raw<F>(f: F, span: tracing::Span) -> Result<Response, ApiError>
where
    F: std::future::Future<Output = Result<Response, ApiError>>,
{
    let future = async {
        let outcome = f.await;
        match outcome.as_ref() {
            Ok(response) => {
                if let Some(content_type) = response.headers().get(CONTENT_TYPE) {
                    if let Ok(content_type) = content_type.to_str() {
                        span.record("resp.type", content_type);
                    }
                }
            }
            Err(e) => {
                span.record("error", true);
                span.record("exception", e.to_string());
                tracing::warn!(target: "exception", name = "the-exception", "{}", e);
            }
        }
        outcome
    };
    future.instrument(span.clone()).await
}

/// Send request, and return unparsed response
/// - req: the request to send
/// - logger: helper to log messages
async fn send_and_unparse(mut req: RequestBuilder, logger: Logger) -> ApiResult<Response> {
    let extensions = req.extensions();

    // Mock
    if let Some(mock) = extensions.get::<MockServer>().cloned() {
        let req = req.build().map_err(ApiError::BuildRequest)?;
        logger.log_mock_request_and_response(&req, mock.type_name());
        let url = req.url().clone();
        match mock.handle(req).await {
            Ok(body) => {
                logger.log_mock_response_body(&body);
                let (content_type, text) = match body {
                    ResponseBody::Empty => (MimeType::Empty, "".to_string()),
                    ResponseBody::Json(json) => (MimeType::Json, json.to_string()),
                    ResponseBody::Xml(xml) => (MimeType::Xml, xml),
                    ResponseBody::Text(text) => (MimeType::Text, text),
                };
                let res = hyper::Response::builder()
                    .url(url)
                    .header(CONTENT_TYPE, content_type.to_string())
                    .body(text)
                    .map_err(|_| {
                        ApiError::Middleware(anyhow::format_err!("Failed to build response"))
                    })?;
                return Ok(Response::from(res));
            }
            Err(e) => {
                logger.log_error(&e);
                return Err(ApiError::Middleware(e));
            }
        }
    }

    let res = req.send().await?;
    Ok(res)
}

/// Send request, and parse response as desired type
/// - req: the request to send
/// - logger: helper to log messages
/// - require_headers: should zip headers into response body
async fn send_and_parse(
    mut req: RequestBuilder,
    logger: Logger,
    require_headers: bool,
) -> ApiResult<ResponseBody> {
    let extensions = req.extensions();

    // Mock
    if let Some(mock) = extensions.get::<MockServer>().cloned() {
        let req = req.build().map_err(ApiError::BuildRequest)?;
        logger.log_mock_request_and_response(&req, mock.type_name());
        match mock.handle(req).await {
            Ok(body) => {
                logger.log_mock_response_body(&body);
                return Ok(body);
            }
            Err(e) => {
                logger.log_error(&e);
                return Err(ApiError::Middleware(e));
            }
        }
    }

    // Send the request
    let res = req.send().await?;

    // Check status code
    let status = res.status();
    let res = if status.is_client_error() || status.is_server_error() {
        let e = if status.is_client_error() {
            ApiError::HttpClientStatus(status.as_u16(), status.to_string())
        } else {
            ApiError::HttpServerStatus(status.as_u16(), status.to_string())
        };
        logger.log_error(&e);
        return Err(e);
    } else {
        res
    };

    // Ignore all payload for 204 No Content
    if res.status() == StatusCode::NO_CONTENT {
        return Ok(ResponseBody::Empty);
    }

    // Check content-type, and parse payload
    let content_type = res
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(MimeType::from)
        .unwrap_or(MimeType::Text);
    match content_type {
        MimeType::Json => parse_as_json(res, content_type, logger, require_headers).await,
        MimeType::Xml => parse_as_xml(res, content_type, logger).await,
        MimeType::Text => parse_as_text(res, content_type, logger).await,
        _ => Err(ApiError::UnsupportedContentType(content_type)),
    }
}

/// Parse response body to json
async fn parse_as_json(
    res: Response,
    content_type: MimeType,
    logger: Logger,
    require_headers: bool,
) -> ApiResult<ResponseBody> {
    // Extract HTTP headers from response
    let headers = if require_headers {
        let mut headers = HashMap::new();
        for (name, value) in res.headers() {
            if let Ok(value) = value.to_str() {
                headers.insert(name.to_string(), value.to_string());
            }
        }
        Some(headers)
    } else {
        None
    };

    // Decode response
    let mut json = match res.json::<Value>().await {
        Ok(json) => {
            logger.log_response_json(&json);
            json
        }
        Err(e) => {
            let e = ApiError::DecodeResponse(content_type, e.to_string());
            logger.log_error(&e);
            return Err(e);
        }
    };

    // Inject headers as `__headers__` field into payload
    // Extractor could parse the `__headers__` field if required
    if let Some(headers) = headers {
        if let Value::Object(m) = &mut json {
            if let Ok(headers) = serde_json::to_value(headers) {
                m.insert("__headers__".to_string(), headers);
            }
        }
    }

    Ok(ResponseBody::Json(json))
}

/// Parse response body to xml
async fn parse_as_xml(
    res: Response,
    content_type: MimeType,
    logger: Logger,
) -> ApiResult<ResponseBody> {
    // Decode response as text
    let text = match res.text().await {
        Ok(text) => {
            logger.log_response_xml(&text);
            text
        }
        Err(e) => {
            let e = ApiError::DecodeResponse(content_type, e.to_string());
            logger.log_error(&e);
            return Err(e);
        }
    };

    Ok(ResponseBody::Xml(text))
}

/// Parse response body to text
async fn parse_as_text(
    res: Response,
    content_type: MimeType,
    logger: Logger,
) -> ApiResult<ResponseBody> {
    // Decode response
    let text = match res.text().await {
        Ok(text) => {
            logger.log_response_text(&text);
            text
        }
        Err(e) => {
            let e = ApiError::DecodeResponse(content_type, e.to_string());
            logger.log_error(&e);
            return Err(e);
        }
    };

    Ok(ResponseBody::Text(text))
}
