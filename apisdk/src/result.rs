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
}

/// Api Error
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Route error: {0}")]
    Router(#[from] RouteError),
    /// Reqwest error
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// Middleware error
    #[error("Middleware error: {0}")]
    Middleware(#[from] anyhow::Error),
    /// Serde(Json) error
    #[error("Serde(Json) error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    /// Invalid json
    #[error("Invalid json: {0}")]
    InvalidJson(Value),
    /// Not multipart form
    #[error("Not multipart form")]
    NotMultipartForm,
    /// Business error
    #[error("Business error: {0} - {1:?}")]
    BusinessError(i64, Option<String>),
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
