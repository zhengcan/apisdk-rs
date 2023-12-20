use std::str::FromStr;

use crate::{ApiError, ApiResult, ResponseBody};

/// This struct is used to parse response body to text
#[derive(Debug)]
pub struct Text;

impl Text {
    /// Try to parse response
    pub fn try_parse<T>(body: ResponseBody) -> ApiResult<T>
    where
        T: FromStr,
    {
        let text = match body {
            ResponseBody::Json(json) => json.to_string(),
            ResponseBody::Xml(xml) => xml,
            ResponseBody::Text(text) => text,
        };
        T::from_str(&text).map_err(|_| ApiError::DecodeText)
    }
}
