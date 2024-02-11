use serde::de::DeserializeOwned;

use crate::{ApiResult, Json, ResponseBody, Xml};

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
            ResponseBody::Json(_) => Json::try_parse(body),
            ResponseBody::Xml(_) => Xml::try_parse(body),
            ResponseBody::Text(_) => {
                Json::try_parse(body.clone()).or_else(|_| Xml::try_parse(body))
            }
        }
    }
}
