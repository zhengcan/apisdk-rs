use hyper::header::HeaderValue;
use serde::de::DeserializeOwned;
use serde_json::Value;

mod auto;
mod json;
mod text;
mod xml;

pub use auto::*;
pub use json::*;
pub use text::*;
pub use xml::*;

use crate::{ApiError, ApiResult};

/// MimeType (aka. ContentType)
#[derive(Debug)]
pub enum MimeType {
    /// Json (application/json)
    Json,
    /// Xml (application/xml | text/xml)
    Xml,
    /// Text (text/plain | text/*)
    Text,
    /// Other
    Other(String),
}

impl std::fmt::Display for MimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "application/json"),
            Self::Xml => write!(f, "application/xml"),
            Self::Text => write!(f, "text/plain"),
            Self::Other(v) => write!(f, "{}", v),
        }
    }
}

impl From<&str> for MimeType {
    fn from(value: &str) -> Self {
        let value = match value.split_once(';') {
            Some((left, _)) => left,
            _ => value.as_ref(),
        }
        .trim()
        .to_lowercase();

        if value == "application/json" {
            Self::Json
        } else if value == "text/xml" || value == "application/xml" {
            Self::Xml
        } else if value.starts_with("text/") {
            Self::Text
        } else {
            Self::Other(value)
        }
    }
}

impl From<MimeType> for HeaderValue {
    fn from(value: MimeType) -> Self {
        HeaderValue::from_str(value.to_string().as_str())
            .unwrap_or(HeaderValue::from_static("text/plain"))
    }
}

/// This enum represents the payload of respones
#[derive(Debug)]
pub enum ResponseBody {
    /// Json (content-type = application/json)
    Json(Value),
    /// Xml (content-type = text/xml | application/xml)
    Xml(String),
    /// Text (content-type = text/plain | text/html | text/*)
    Text(String),
}

impl ResponseBody {
    /// Get the related mime type
    pub fn mime_type(&self) -> MimeType {
        match self {
            Self::Json(_) => MimeType::Json,
            Self::Xml(_) => MimeType::Xml,
            Self::Text(_) => MimeType::Text,
        }
    }

    /// Parse json to target type
    pub fn parse_json<T>(self) -> ApiResult<T>
    where
        T: DeserializeOwned,
    {
        match self {
            Self::Json(json) => serde_json::from_value(json).map_err(ApiError::DecodeJson),
            _ => Err(ApiError::IncompatibleContentType(
                MimeType::Json,
                self.mime_type(),
            )),
        }
    }

    /// Parse json to target type
    pub fn parse_xml<T>(self) -> ApiResult<T>
    where
        T: DeserializeOwned,
    {
        match self {
            Self::Xml(xml) => quick_xml::de::from_str(&xml).map_err(ApiError::DecodeXml),
            Self::Text(text) => {
                log::debug!("Treat text as xml for decoding");
                quick_xml::de::from_str(&text).map_err(ApiError::DecodeXml)
            }
            _ => Err(ApiError::IncompatibleContentType(
                MimeType::Xml,
                self.mime_type(),
            )),
        }
    }
}

/// This struct is used to parse response body to xml
#[derive(Debug)]
pub struct Body;

impl Body {
    /// Try to parse response
    pub fn try_parse<T>(body: ResponseBody) -> ApiResult<T>
    where
        T: TryFrom<ResponseBody>,
        T::Error: Into<ApiError>,
    {
        T::try_from(body).map_err(|e| e.into())
    }
}
