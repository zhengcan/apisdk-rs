use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{ApiError, ApiResult, Json, ResponseBody, Xml};

use super::MimeType;

/// This struct is used to parse response body to json or xml
#[derive(Debug)]
pub struct Auto;

impl Auto {
    /// Try to parse response
    pub fn try_parse<T>(body: ResponseBody) -> ApiResult<T>
    where
        T: 'static + DeserializeOwned,
    {
        match &body {
            ResponseBody::Empty => serde_json::from_value(Value::Null).map_err(|_| {
                ApiError::DecodeResponse(
                    MimeType::Empty,
                    "Failed to decode empty response to result type.".to_string(),
                )
            }),
            ResponseBody::Json(_) => Json::try_parse(body),
            ResponseBody::Xml(_) => Xml::try_parse(body),
            ResponseBody::Text(_) => {
                Json::try_parse(body.clone()).or_else(|_| Xml::try_parse(body))
            }
        }
    }
}
