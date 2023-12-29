use std::any::TypeId;

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{ApiError, ApiResult, MimeType, ResponseBody};

/// This struct is used to parse response body to xml
#[derive(Debug)]
pub struct Xml;

impl Xml {
    fn do_try_parse<T>(text: String) -> ApiResult<T>
    where
        T: 'static + DeserializeOwned,
    {
        let type_id = TypeId::of::<T>();
        if type_id == TypeId::of::<()>() {
            serde_json::from_value(Value::Null)
                .map_err(|_| ApiError::Other("Impossible".to_string()))
        } else if type_id == TypeId::of::<String>() {
            let value = serde_json::Value::String(text);
            serde_json::from_value(value).map_err(|_| ApiError::Other("Impossible".to_string()))
        } else {
            quick_xml::de::from_str(&text).map_err(ApiError::DecodeXml)
        }
    }

    /// Try to parse response
    pub fn try_parse<T>(body: ResponseBody) -> ApiResult<T>
    where
        T: 'static + DeserializeOwned,
    {
        let type_id = TypeId::of::<T>();
        if type_id == TypeId::of::<()>() {
            return serde_json::from_value(Value::Null)
                .map_err(|_| ApiError::Other("Impossible".to_string()));
        }

        match body {
            ResponseBody::Xml(xml) => Self::do_try_parse(xml),
            ResponseBody::Text(text) => {
                log::debug!("Treat text as xml for decoding");
                Self::do_try_parse(text)
            }
            _ => Err(ApiError::IncompatibleContentType(
                MimeType::Xml,
                body.mime_type(),
            )),
        }
    }
}
