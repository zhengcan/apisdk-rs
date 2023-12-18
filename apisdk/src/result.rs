use reqwest::Url;
use serde_json::Value;
use thiserror::Error;

use crate::MiddlewareError;

/// Route Error
#[derive(Debug, Error)]
pub enum RouteError {
    /// Service discovery error
    #[error("Service discovery error: {0}")]
    ServiceDiscovery(anyhow::Error),
    /// Update scheme error
    #[error("Update scheme error: {0} => {1}")]
    UpdateScheme(Url, String),
    /// Update host error
    #[error("Update host error: {0} => {1}")]
    UpdateHost(Url, String, url::ParseError),
    /// Update port error
    #[error("Update port error: {0} => :{1}")]
    UpdatePort(Url, u16),
    /// Custom error
    #[error("Custom error: {0}")]
    Custom(String),
}

/// Api Error
#[derive(Debug, Error)]
pub enum ApiError {
    /// Route error
    #[error("Route error: {0}")]
    Route(#[from] RouteError),
    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(reqwest::Error),
    /// Build request error
    #[error("Build request error: {0}")]
    BuildRequest(reqwest::Error),
    /// Generic reqwest error
    #[error("Generic reqwest error: {0}")]
    Reqwest(reqwest::Error),
    /// Middleware error
    #[error("Middleware error: {0}")]
    Middleware(anyhow::Error),
    /// Invalid multipart form
    #[error("Invalid multipart form")]
    MultipartForm,
    /// HTTP Client status error
    #[error("HTTP Client status error: [{0}] {1}")]
    HttpClientStatus(u16, String),
    /// HTTP Server status error
    #[error("HTTP Server status error: [{0}] {1}")]
    HttpServerStatus(u16, String),
    /// Unsupported Content-Type
    #[error("Unsupported Content-Type: {0}")]
    UnsupportedContentType(String),
    /// Decode response error
    #[error("Decode response error: {0} => {1}")]
    DecodeResponse(String, String),
    /// Decode json error
    #[error("Decode json error: {0}")]
    DecodeJson(#[from] serde_json::Error),
    /// Invalid json
    #[error("Invalid json: {0}")]
    InvalidJson(Value),
    /// Business error
    #[error("Business error: {0} - {1:?}")]
    BusinessError(i64, Option<String>),
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_status() {
            let status = e.status().unwrap_or_default();
            if status.is_client_error() {
                ApiError::HttpClientStatus(status.as_u16(), status.to_string())
            } else {
                ApiError::HttpServerStatus(status.as_u16(), status.to_string())
            }
        } else {
            ApiError::Reqwest(e)
        }
    }
}

impl From<MiddlewareError> for ApiError {
    fn from(e: MiddlewareError) -> Self {
        match e {
            MiddlewareError::Reqwest(e) => Self::Reqwest(e),
            MiddlewareError::Middleware(e) => Self::Middleware(e),
        }
    }
}

/// An alias of Result<T, ApiError
pub type ApiResult<T> = Result<T, ApiError>;
