use std::any::TypeId;

use serde::de::DeserializeOwned;

use crate::{ApiError, ApiResult, ResponseBody};

/// This struct is used to parse response body to xml
#[derive(Debug)]
pub struct Xml;

impl Xml {
    /// Try to parse response
    pub fn try_parse<T>(body: ResponseBody) -> ApiResult<T>
    where
        T: 'static + DeserializeOwned,
    {
        match body {
            ResponseBody::Xml(xml) => {
                let type_id = TypeId::of::<T>();
                if type_id == TypeId::of::<String>() {
                    let value = serde_json::Value::String(xml);
                    serde_json::from_value(value).map_err(|_| ApiError::Other)
                } else {
                    quick_xml::de::from_str(&xml).map_err(ApiError::DecodeXml)
                }
            }
            _ => Err(ApiError::Other),
        }
    }
}
