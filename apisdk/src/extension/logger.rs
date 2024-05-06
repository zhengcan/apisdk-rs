use std::{collections::HashMap, str::FromStr, sync::OnceLock, time::Instant};

use async_trait::async_trait;
use http::Extensions;
use lazy_static::lazy_static;
use log::{Level, LevelFilter};
use regex::Regex;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next, RequestBuilder, RequestInitialiser};
use serde_json::Value;

use crate::ResponseBody;

static DEFAULT_LOG_LEVEL: OnceLock<LevelFilter> = OnceLock::new();

/// Set the log level as global default
pub fn init_default_log_level(level: LevelFilter) -> Result<(), LevelFilter> {
    DEFAULT_LOG_LEVEL.set(level)
}

pub(crate) fn get_default_log_level() -> LevelFilter {
    *DEFAULT_LOG_LEVEL.get_or_init(|| LevelFilter::Debug)
}

/// This trait is used to create `LevelFilter`
pub trait IntoFilter {
    fn into_filter(self) -> Option<LevelFilter>;
}

impl IntoFilter for bool {
    fn into_filter(self) -> Option<LevelFilter> {
        if self {
            Some(get_default_log_level())
        } else {
            Some(LevelFilter::Off)
        }
    }
}

impl IntoFilter for &str {
    fn into_filter(self) -> Option<LevelFilter> {
        LevelFilter::from_str(self).ok()
    }
}

impl IntoFilter for LevelFilter {
    fn into_filter(self) -> Option<LevelFilter> {
        Some(self)
    }
}

impl IntoFilter for Level {
    fn into_filter(self) -> Option<LevelFilter> {
        Some(self.to_level_filter())
    }
}

/// This struct is used to control how to log.
/// It could be injected into request as an extension.
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Level filter
    pub level: LevelFilter,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: get_default_log_level(),
        }
    }
}

impl LogConfig {
    /// Construct a new instance
    pub fn new<L>(level: L) -> Self
    where
        L: IntoFilter,
    {
        Self {
            level: level.into_filter().unwrap_or(get_default_log_level()),
        }
    }

    /// Construct a new instance to turn off logs
    pub fn off() -> Self {
        Self {
            level: LevelFilter::Off,
        }
    }
}

impl RequestInitialiser for LogConfig {
    fn init(&self, req: RequestBuilder) -> RequestBuilder {
        let mut req = req;
        match req.extensions().get::<LogConfig>() {
            Some(_) => req,
            None => req.with_extension(self.clone()),
        }
    }
}

/// This middleware is used to write logs
pub(crate) struct LogMiddleware;

#[async_trait]
impl Middleware for LogMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, reqwest_middleware::Error> {
        match extensions.remove::<Logger>() {
            Some(logger) => {
                logger.log_request(&req);
                let res = next.run(req, extensions).await?;
                logger.log_response(&res);
                Ok(res)
            }
            None => next.run(req, extensions).await,
        }
    }
}

/// This enum is used to hold request payload for logging
#[derive(Debug, Clone)]
enum RequestPayload {
    Json(Value),
    Xml(String),
    Form(HashMap<String, String>),
    Multipart(HashMap<String, String>),
}

/// This struct is used to write information to log
#[derive(Debug, Clone)]
pub(crate) struct Logger {
    /// The target of log
    log_target: String,
    /// The level of log
    log_level: Option<Level>,
    /// The X-Request-ID value
    request_id: String,
    /// The start instant
    start: Instant,
    /// The request payload
    payload: Option<RequestPayload>,
}

lazy_static! {
    static ref REGEX: Regex = Regex::new(r"<impl (.+::)*(.*)>").unwrap();
}

impl Logger {
    /// Create a new instance
    pub fn new(log_target: &'static str, log_filter: LevelFilter, request_id: String) -> Self {
        Self {
            log_target: REGEX.replace_all(log_target, "<$2>").to_string(),
            log_level: log_filter.to_level(),
            request_id,
            start: Instant::now(),
            payload: None,
        }
    }

    /// Check the log is enabled or not
    pub fn is_enabled(&self) -> bool {
        self.log_level.is_some()
    }

    /// Extends with json payload
    pub fn with_json(mut self, json: Value) -> Self {
        self.payload = Some(RequestPayload::Json(json));
        self
    }

    /// Extends with xml payload
    pub fn with_xml(mut self, xml: String) -> Self {
        self.payload = Some(RequestPayload::Xml(xml));
        self
    }

    /// Extends with form payload
    pub fn with_form(mut self, meta: HashMap<String, String>) -> Self {
        self.payload = Some(RequestPayload::Form(meta));
        self
    }

    /// Extends with multipart form payload
    pub fn with_multipart(mut self, meta: HashMap<String, String>) -> Self {
        self.payload = Some(RequestPayload::Multipart(meta));
        self
    }
}

impl Logger {
    /// Log request
    pub fn log_request(&self, req: &Request) {
        if let Some(level) = self.log_level {
            log::log!(target: &self.log_target, level, "#[{}] {:?}", self.request_id, req);
            if let Some(payload) = self.payload.as_ref() {
                self.log_request_payload(level, payload);
            }
        }
    }

    fn log_request_payload(&self, level: Level, payload: &RequestPayload) {
        match payload {
            RequestPayload::Json(json) => {
                log::log!(target: &self.log_target, level, "#[{}] Request Json\n{}", self.request_id, json);
            }
            RequestPayload::Xml(xml) => {
                log::log!(target: &self.log_target, level, "#[{}] Request Xml\n{:?}", self.request_id, xml);
            }
            RequestPayload::Form(meta) => {
                log::log!(target: &self.log_target, level, "#[{}] Request Form\n{:?}", self.request_id, meta);
            }
            RequestPayload::Multipart(meta) => {
                log::log!(target: &self.log_target, level, "#[{}] Request Multipart\n{:?}", self.request_id, meta);
            }
        }
    }

    /// Log response
    pub fn log_response(&self, res: &Response) {
        if let Some(level) = self.log_level {
            log::log!(
                target: &self.log_target,
                level,
                "#[{}] {:?} @{}ms",
                self.request_id,
                res,
                self.start.elapsed().as_millis()
            );
        }
    }

    /// Log response json payload
    pub fn log_response_json(&self, json: &Value) {
        if let Some(level) = self.log_level {
            log::log!(
                target: &self.log_target,
                level,
                "#[{}] Response Body(Json) @{}ms\n{}",
                self.request_id,
                self.start.elapsed().as_millis(),
                serde_json::to_string(json).unwrap_or_default()
            );
        }
    }

    /// Log response xml payload
    pub fn log_response_xml(&self, xml: &str) {
        if let Some(level) = self.log_level {
            log::log!(
                target: &self.log_target,
                level,
                "#[{}] Response Body(Xml) @{}ms\n{}",
                self.request_id,
                self.start.elapsed().as_millis(),
                &xml[0..1024.min(xml.len())]
            );
        }
    }

    /// Log response text payload
    pub fn log_response_text(&self, text: &str) {
        if let Some(level) = self.log_level {
            log::log!(
                target: &self.log_target,
                level,
                "#[{}] Response Body(Text) @{}ms\n{}",
                self.request_id,
                self.start.elapsed().as_millis(),
                &text[0..1024.min(text.len())]
            );
        }
    }

    /// Log mock request and response
    pub fn log_mock_request_and_response(&self, req: &Request, mock_name: &str) {
        if let Some(level) = self.log_level {
            log::log!(target: &self.log_target, level, "#[{}] {:?}", self.request_id, req);
            log::log!(target: &self.log_target, level, "#[{}] Response (MOCK) <= {}", self.request_id, mock_name);
        }
    }

    /// Log mock response body
    pub fn log_mock_response_body(&self, body: &ResponseBody) {
        match body {
            ResponseBody::Json(json) => self.log_response_json(json),
            ResponseBody::Xml(xml) => self.log_response_xml(xml),
            ResponseBody::Text(text) => self.log_response_text(text),
        }
    }

    /// Log error as warn or higher level
    pub fn log_error(&self, e: impl std::fmt::Display) {
        let level = self.log_level.unwrap_or(Level::Debug).min(Level::Warn);
        log::log!(
            target: &self.log_target,
            level,
            "#[{}] Error @{}ms: {}",
            self.request_id,
            self.start.elapsed().as_millis(),
            e
        );
    }
}
