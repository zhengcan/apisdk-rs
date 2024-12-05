use serde_json::Value;
use thiserror::Error;

use crate::{MiddlewareError, MimeType};

/// Api Error
#[derive(Debug, Error)]
pub enum ApiError {
    /// Service discovery error
    #[error("Service discovery error: {0}")]
    ServiceDiscovery(anyhow::Error),
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
    UnsupportedContentType(MimeType),
    /// Incompatible Content-Type
    #[error("Incompatible Content-Type: perfer {0}, actual {1}")]
    IncompatibleContentType(MimeType, MimeType),
    /// Decode response error
    /// - 0: value of content-type
    /// - 1: message
    #[error("Decode response error: {0} => {1}")]
    DecodeResponse(MimeType, String),
    /// Decode json error
    #[error("Decode json error: {0}")]
    DecodeJson(#[from] serde_json::Error),
    /// Decode xml error
    #[error("Decode xml error: {0}")]
    DecodeXml(#[from] quick_xml::DeError),
    /// Decode text error
    #[error("Decode text error")]
    DecodeText,
    /// Illegal json
    #[error("Illegal json: {0}")]
    IllegalJson(Value),
    /// Decode xml error
    #[error("Illegal xml: {0}")]
    IllegalXml(#[from] quick_xml::SeError),
    /// Service error
    #[error("Service error: {0} - {1:?}")]
    ServiceError(i64, Option<String>),
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
    /// Impossible
    #[error("It's impossible here.")]
    Impossible,
}

impl ApiError {
    /// Build ApiError by using `code` and `message`
    pub fn new(code: i64, message: impl ToString) -> Self {
        Self::ServiceError(code, Some(message.to_string()))
    }

    /// Try to retrieve `error_code`
    pub fn as_error_code(&self) -> i32 {
        match self {
            Self::ServiceDiscovery(..)
            | Self::InvalidUrl(..)
            | Self::BuildRequest(..)
            | Self::Reqwest(..)
            | Self::Middleware(..)
            | Self::MultipartForm => 400,
            Self::HttpClientStatus(c, _) => *c as i32,
            Self::HttpServerStatus(c, _) => *c as i32,
            Self::UnsupportedContentType(..)
            | Self::IncompatibleContentType(..)
            | Self::DecodeResponse(..)
            | Self::DecodeJson(..)
            | Self::DecodeXml(..)
            | Self::DecodeText
            | Self::IllegalJson(..)
            | Self::IllegalXml(..) => 500,
            Self::ServiceError(c, _) => *c as i32,
            Self::Other(..) | Self::Impossible => 500,
        }
    }
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
