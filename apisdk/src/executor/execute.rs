use std::collections::HashMap;

use reqwest::{header::CONTENT_TYPE, Response, ResponseBuilderExt};
use serde::Serialize;
use serde_json::Value;

use crate::{
    ApiError, ApiResult, FormLike, LogConfig, MockServer, RequestBuilder, RequestId,
    RequestTraceIdInjector, Responder, ResponseBody,
};

/// This struct is used to build RequestConfig internally by macros.
#[derive(Debug, Default)]
pub struct RequestConfigurator {
    /// The target of log
    log_target: &'static str,
    /// Indicate whether to log
    log_enabled: Option<bool>,
    /// Indicate whether to parse headers from response or not
    require_headers: bool,
}

impl RequestConfigurator {
    /// Create a new instance
    pub fn new(log_target: &'static str, log_enabled: Option<bool>, require_headers: bool) -> Self {
        Self {
            log_target,
            log_enabled,
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

    /// Build RequestConfig
    fn build(self, req: &mut RequestBuilder) -> RequestConfig {
        let extensions = req.extensions();

        let request_id = extensions
            .get::<RequestId>()
            .map(|id| id.request_id.clone())
            .unwrap_or_default();

        let log_enabled = extensions
            .get::<LogConfig>()
            .map(|log_config| log_config.enabled)
            .unwrap_or_default();
        RequestConfig {
            log_target: self.log_target,
            log_enabled: self.log_enabled.unwrap_or(log_enabled),
            require_headers: self.require_headers,
            request_id,
        }
    }
}

/// This config is used to control the send process
#[derive(Debug, Default)]
struct RequestConfig {
    /// The target of log
    log_target: &'static str,
    /// Indicate whether to log
    log_enabled: bool,
    /// Indicate whether to parse headers from response or not
    require_headers: bool,
    /// The X-Request-ID value
    request_id: String,
}

impl RequestConfig {
    pub fn request_id(&self) -> &str {
        self.request_id.as_str()
    }
}

/// Send request
/// - req: used to build request
/// - config: control the send process
pub async fn _send(
    mut req: RequestBuilder,
    config: RequestConfigurator,
) -> ApiResult<ResponseBody> {
    req = RequestTraceIdInjector::inject_extension(req);

    let config = config.build(&mut req);
    if config.log_enabled {
        log::debug!(target: config.log_target, "#[{}] {:?}", config.request_id(), req);
    }

    send_and_parse(req, config).await
}

/// Send request with JSON payload
/// - req: used to build request
/// - json: request payload
/// - config: control the send process
pub async fn _send_json<I>(
    mut req: RequestBuilder,
    json: &I,
    config: RequestConfigurator,
) -> ApiResult<ResponseBody>
where
    I: Serialize + ?Sized,
{
    req = RequestTraceIdInjector::inject_extension(req);

    req = req.json(json);

    let config = config.build(&mut req);
    if config.log_enabled {
        log::debug!(target: config.log_target, "#[{}] {:?}", config.request_id(), req);
        log::debug!(target: config.log_target, "#[{}] Json {}", config.request_id(), serde_json::to_string(json).unwrap_or_default());
    }

    send_and_parse(req, config).await
}

/// Send request with form payload
/// - req: used to build request
/// - form: request payload
/// - config: control the send process
pub async fn _send_form<I>(
    mut req: RequestBuilder,
    form: I,
    config: RequestConfigurator,
) -> ApiResult<ResponseBody>
where
    I: FormLike,
{
    req = RequestTraceIdInjector::inject_extension(req);

    let is_multipart = form.is_multipart();
    let meta = form.get_meta();

    if is_multipart {
        if let Some(multipart) = form.get_multipart() {
            req = req.multipart(multipart)
        }
    } else if let Some(form) = form.get_form() {
        req = req.form(&form);
    };

    let config = config.build(&mut req);
    if config.log_enabled {
        log::debug!(target: config.log_target, "#[{}] {:?}", config.request_id(), req);
        log::debug!(target: config.log_target, "#[{}] {} {:?}", config.request_id(), if is_multipart { "Multipart"} else {"Form"}, meta);
    }

    send_and_parse(req, config).await
}

/// Send request with multipart/data payload
/// - req: used to build request
/// - form: request payload
/// - config: control the send process
pub async fn _send_multipart<I>(
    mut req: RequestBuilder,
    form: I,
    config: RequestConfigurator,
) -> ApiResult<ResponseBody>
where
    I: FormLike,
{
    req = RequestTraceIdInjector::inject_extension(req);

    let form = form.get_multipart().ok_or(ApiError::MultipartForm)?;
    req = req.multipart(form);

    let config = config.build(&mut req);
    if config.log_enabled {
        log::debug!(target: config.log_target, "#[{}] {:?}", config.request_id(), req);
    }

    send_and_parse(req, config).await
}

/// Send request, and get raw response
/// - req: used to build request
/// - config: control the send process
pub async fn _send_raw(
    mut req: RequestBuilder,
    config: RequestConfigurator,
) -> ApiResult<Response> {
    req = RequestTraceIdInjector::inject_extension(req);

    let config = config.build(&mut req);
    if config.log_enabled {
        log::debug!(target: config.log_target, "#[{}] {:?}", config.request_id(), req);
    }

    send_and_unparse(req, config).await
}

/// Send request, and return unparsed response
/// - req: the request to send
/// - config: control the send process
async fn send_and_unparse(mut req: RequestBuilder, config: RequestConfig) -> ApiResult<Response> {
    let extensions = req.extensions();

    // Mock
    if let Some(mock) = extensions.get::<MockServer>().cloned() {
        let req = req.build().map_err(ApiError::BuildRequest)?;
        if config.log_enabled {
            log::debug!(target: config.log_target, "#[{}] Response (MOCK)", config.request_id());
        }
        let url = req.url().clone();
        match mock.handle(req).await {
            Ok(body) => {
                if config.log_enabled {
                    match &body {
                        ResponseBody::Json(json) => {
                            log::debug!(target: config.log_target, "#[{}] Payload {}", config.request_id(), serde_json::to_string(json).unwrap_or_default());
                        }
                        ResponseBody::Text(text) => {
                            log::debug!(target: config.log_target, "#[{}] Payload(Text) {}", config.request_id(), text);
                        }
                    }
                }

                let (content_type, text) = match body {
                    ResponseBody::Json(json) => ("application/json", json.to_string()),
                    ResponseBody::Text(text) => ("text/plain", text),
                };
                let res = hyper::Response::builder()
                    .url(url)
                    .header(CONTENT_TYPE, content_type)
                    .body(text)
                    .map_err(|_| {
                        ApiError::Middleware(anyhow::format_err!("Failed to build response"))
                    })?;
                return Ok(Response::from(res));
            }
            Err(e) => {
                if config.log_enabled {
                    log::debug!(target: config.log_target, "#[{}] Error: {}", config.request_id(), e);
                }
                return Err(ApiError::Middleware(e));
            }
        }
    }

    let res = req.send().await?;
    if config.log_enabled {
        log::debug!(target: config.log_target, "#[{}] {:?}", config.request_id(), res);
    }

    Ok(res)
}

/// Send request, and parse response as desired type
/// - req: the request to send
/// - config: control the send process
async fn send_and_parse(mut req: RequestBuilder, config: RequestConfig) -> ApiResult<ResponseBody> {
    let extensions = req.extensions();

    // Mock
    if let Some(mock) = extensions.get::<MockServer>().cloned() {
        let req = req.build().map_err(ApiError::BuildRequest)?;
        if config.log_enabled {
            log::debug!(target: config.log_target, "#[{}] Response (MOCK)", config.request_id());
        }
        match mock.handle(req).await {
            Ok(body) => {
                if config.log_enabled {
                    match &body {
                        ResponseBody::Json(json) => {
                            log::debug!(target: config.log_target, "#[{}] Payload {}", config.request_id(), serde_json::to_string(json).unwrap_or_default());
                        }
                        ResponseBody::Text(text) => {
                            log::debug!(target: config.log_target, "#[{}] Payload(Text) {}", config.request_id(), text);
                        }
                    }
                }
                return Ok(body);
            }
            Err(e) => {
                if config.log_enabled {
                    log::debug!(target: config.log_target, "#[{}] Error: {}", config.request_id(), e);
                }
                return Err(ApiError::Middleware(e));
            }
        }
    }

    // Send the request
    let res = req.send().await?;
    if config.log_enabled {
        log::debug!(target: config.log_target, "#[{}] {:?}", config.request_id(), res);
    }

    // Check status code
    let status = res.status();
    let res = if status.is_client_error() || status.is_server_error() {
        let e = if status.is_client_error() {
            ApiError::HttpClientStatus(status.as_u16(), status.to_string())
        } else {
            ApiError::HttpServerStatus(status.as_u16(), status.to_string())
        };
        if config.log_enabled {
            log::debug!(target: config.log_target, "#[{}] Error: {}", config.request_id(), e);
        }
        return Err(e);
    } else {
        res
    };

    // Check content-type, and parse payload
    let content_type = res
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("text/plain")
        .to_lowercase();
    if content_type.starts_with("application/json") {
        parse_as_json(res, content_type, config).await
    } else if content_type.starts_with("text/") {
        parse_as_text(res, content_type, config).await
    } else {
        return Err(ApiError::IllegalContentType(content_type));
    }
}

async fn parse_as_json(
    res: Response,
    content_type: String,
    config: RequestConfig,
) -> ApiResult<ResponseBody> {
    // Extract HTTP headers from response
    let headers = if config.require_headers {
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

    // Parse payload as json
    let mut json = match res.json::<Value>().await {
        Ok(json) => {
            if config.log_enabled {
                log::debug!(target: config.log_target, "#[{}] Body: {}", config.request_id(), serde_json::to_string(&json).unwrap_or_default());
            }
            json
        }
        Err(e) => {
            let e = ApiError::DecodeResponse(content_type, e.to_string());
            if config.log_enabled {
                log::debug!(target: config.log_target, "#[{}] Error: {}", config.request_id(), e);
            }
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

    // Deserialize payload
    match serde_json::from_value(json) {
        Ok(v) => Ok(ResponseBody::Json(v)),
        Err(e) => {
            let e = ApiError::DecodeJson(e);
            if config.log_enabled {
                log::debug!(target: config.log_target, "#[{}] Error: {}", config.request_id(), e);
            }
            Err(e)
        }
    }
}

async fn parse_as_text(
    res: Response,
    content_type: String,
    config: RequestConfig,
) -> ApiResult<ResponseBody> {
    let text = match res.text().await {
        Ok(text) => text,
        Err(e) => {
            let e = ApiError::DecodeResponse(content_type, e.to_string());
            if config.log_enabled {
                log::debug!(target: config.log_target, "#[{}] Error: {}", config.request_id(), e);
            }
            return Err(e);
        }
    };

    Ok(ResponseBody::Text(text))
}
