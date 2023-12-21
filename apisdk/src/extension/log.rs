use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next, RequestBuilder, RequestInitialiser};
use serde_json::Value;
use task_local_extensions::Extensions;

use crate::ResponseBody;

/// This struct is used to control how to log
#[derive(Debug, Default, Clone)]
pub struct LogConfig {
    /// Enable logs
    pub enabled: bool,
}

impl LogConfig {
    /// Construct a new instance
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Construct a new instance to enable all logs
    pub fn enabled_all() -> Self {
        Self { enabled: true }
    }
}

impl RequestInitialiser for LogConfig {
    fn init(&self, req: RequestBuilder) -> RequestBuilder {
        let mut req = req;
        if req.extensions().contains::<LogConfig>() {
            req
        } else {
            req.with_extension(self.clone())
        }
    }
}

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

#[derive(Debug, Clone)]
enum RequestPayload {
    Json(Value),
    Form(HashMap<String, String>),
    Multipart(HashMap<String, String>),
}

/// This struct is used to write information to log
#[derive(Debug, Clone, Default)]
pub(crate) struct Logger {
    /// The target of log
    pub log_target: &'static str,
    /// Indicate whether to log
    pub log_enabled: bool,
    /// The X-Request-ID value
    pub request_id: String,
    /// The request payload
    payload: Option<RequestPayload>,
}

impl Logger {
    pub fn new(log_target: &'static str, log_enabled: bool, request_id: String) -> Self {
        Self {
            log_target,
            log_enabled,
            request_id,
            payload: None,
        }
    }

    pub fn with_json(mut self, json: Value) -> Self {
        self.payload = Some(RequestPayload::Json(json));
        self
    }

    pub fn with_form(mut self, meta: HashMap<String, String>) -> Self {
        self.payload = Some(RequestPayload::Form(meta));
        self
    }

    pub fn with_multipart(mut self, meta: HashMap<String, String>) -> Self {
        self.payload = Some(RequestPayload::Multipart(meta));
        self
    }
}

impl Logger {
    pub fn log_request(&self, req: &Request) {
        if self.log_enabled {
            log::debug!(target: self.log_target, "#[{}] {:?}", self.request_id, req);
            if let Some(payload) = self.payload.as_ref() {
                self.log_request_payload(payload);
            }
        }
    }

    fn log_request_payload(&self, payload: &RequestPayload) {
        match payload {
            RequestPayload::Json(json) => {
                log::debug!(target: self.log_target, "#[{}] Json\n{}", self.request_id, json);
            }
            RequestPayload::Form(meta) => {
                log::debug!(target: self.log_target, "#[{}] Form\n{:?}", self.request_id, meta);
            }
            RequestPayload::Multipart(meta) => {
                log::debug!(target: self.log_target, "#[{}] Multipart\n{:?}", self.request_id, meta);
            }
        }
    }

    pub fn log_response(&self, res: &Response) {
        if self.log_enabled {
            log::debug!(target: self.log_target, "#[{}] {:?}", self.request_id, res);
        }
    }

    pub fn log_response_json(&self, json: &Value) {
        if self.log_enabled {
            log::debug!(target: self.log_target, "#[{}] Body(Json)\n{}", self.request_id, serde_json::to_string(json).unwrap_or_default());
        }
    }

    pub fn log_response_xml(&self, xml: &str) {
        if self.log_enabled {
            log::debug!(target: self.log_target, "#[{}] Body(Xml)\n{}", self.request_id, &xml[0..1024.min(xml.len())]);
        }
    }

    pub fn log_response_text(&self, text: &str) {
        if self.log_enabled {
            log::debug!(target: self.log_target, "#[{}] Body(Text)\n{}", self.request_id, &text[0..1024.min(text.len())]);
        }
    }

    pub fn log_mock_request_and_response(&self, req: &Request, mock_name: &str) {
        if self.log_enabled {
            log::debug!(target: self.log_target, "#[{}] {:?}", self.request_id, req);
            log::debug!(target: self.log_target, "#[{}] Response (MOCK) <= {}", self.request_id, mock_name);
        }
    }

    pub fn log_mock_response_body(&self, body: &ResponseBody) {
        if self.log_enabled {
            match body {
                ResponseBody::Json(json) => self.log_response_json(json),
                ResponseBody::Xml(xml) => self.log_response_xml(xml),
                ResponseBody::Text(text) => self.log_response_text(text),
            }
        }
    }

    pub fn log_error(&self, e: impl std::fmt::Display) {
        if self.log_enabled {
            log::debug!(target: self.log_target, "#[{}] Error: {}", self.request_id, e);
        }
    }
}
