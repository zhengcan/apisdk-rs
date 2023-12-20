use std::{any::TypeId, collections::HashMap};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::{ApiError, ApiResult};

use super::ResponseBody;

/// This struct is used to parse response body to json
#[derive(Debug)]
pub struct Json;

impl Json {
    /// Try to parse response
    pub fn try_parse<T>(body: ResponseBody) -> ApiResult<T>
    where
        T: 'static + DeserializeOwned,
    {
        match body {
            ResponseBody::Json(json) => {
                let type_id = TypeId::of::<T>();
                if type_id == TypeId::of::<String>() {
                    let value = serde_json::Value::String(json.to_string());
                    serde_json::from_value(value).map_err(|_| ApiError::Other)
                } else {
                    serde_json::from_value(json).map_err(ApiError::DecodeJson)
                }
            }
            _ => Err(ApiError::Other),
        }
    }
}

/// This trait is used to extract result from response.
///
/// # Usage
///
/// ```
/// let req = client.get("/api/path").await?;
/// let res = send!(req, TypeOfExtractor).await?;
/// ```
///
/// # Examples
///
/// ### Check return code
///
/// ```
/// #[derive(serde::Deserialize)]
/// pub struct CheckReturnCode(serde_json::Value);
///
/// impl JsonExtractor for CheckReturnCode {
///     fn try_extract(self) -> ApiResult<T> {
///         match self.0.get("ret_code").and_then(|c| c.as_i64()) {
///             Some(0) => serde_json::from_value(self.0).map_err(|e| e.into()),
///             Some(c) => Err(ApiError::BusinessError(c, Some("Invalid ret_code".to_string()))),
///             None => Err(ApiError::BusinessError(-1, Some("No ret_code".to_string()))),
///         }
///     }
/// }
/// ```
///
/// ### Extract single field
///
/// ```
/// #[derive(serde::Deserialize)]
/// pub struct ExtractData {
///     data: serde_json::Value
/// }
///
/// impl JsonExtractor for ExtractData {
///     fn try_extract(self) -> ApiResult<T> {
///         serde_json::from_value(self.data).map_err(|e| e.into())
///     }
/// }
/// ```
///
/// # Built-in Extractors
///
/// - serde_json::Value
///     - treat whole payload as json output
/// - apisdk::WholePayload
///     - an alias of serde_json::Value
/// - apisdk::CodeDataMessage
///     - parse `{code, data, message}` json payload, and return `data` field
pub trait JsonExtractor {
    /// The extractor needs response HTTP headers or not.
    fn require_headers() -> bool {
        false
    }

    /// Try to extract result from response.
    ///
    /// The HTTP headers will be inject as `__headers__` field if possible.
    fn try_extract<T>(self) -> ApiResult<T>
    where
        T: DeserializeOwned;
}

impl TryFrom<ResponseBody> for Value {
    type Error = ApiError;

    fn try_from(body: ResponseBody) -> Result<Self, Self::Error> {
        body.parse_json()
    }
}

impl JsonExtractor for Value {
    fn try_extract<T>(self) -> ApiResult<T>
    where
        T: DeserializeOwned,
    {
        serde_json::from_value(self).map_err(|_| ApiError::IllegalJson(Value::Null))
    }
}

impl TryFrom<ResponseBody> for String {
    type Error = ApiError;

    fn try_from(body: ResponseBody) -> Result<Self, Self::Error> {
        match body {
            ResponseBody::Json(json) => {
                // Remove __headers__
                let json = match json {
                    Value::Object(mut map) => {
                        map.remove("__headers__");
                        Value::Object(map)
                    }
                    _ => json,
                };
                Ok(json.to_string())
            }
            ResponseBody::Xml(xml) => Ok(xml),
            ResponseBody::Text(text) => Ok(text),
        }
    }
}

impl JsonExtractor for String {
    fn try_extract<T>(self) -> ApiResult<T>
    where
        T: DeserializeOwned,
    {
        serde_json::from_value(Value::String(self)).map_err(|_| ApiError::IllegalJson(Value::Null))
    }
}

/// This extractor will treat whole payload as result
pub type WholePayload = Value;

/// This struct is used to parse `{code, data, message}` payload.
///
/// When it's used as `Extractor`, it will extract `data` from payload.
///
/// # Examples
///
/// ### As Extractor
///
/// To be used as `Extractor`, `CodeDataMessage` will check `code` field of response payload, and ensure it must be `0`.
/// If not, it will generate an ApiError instance with `code` and `message`.
///
/// ```
/// async fn get_user(&self) -> ApiResult<User> {
///     let req = client.get("/api/path").await?;
///     send!(req, CodeDataMessage).await
/// }
/// ```
///
/// ### As Result
///
/// If we want to access the response headers or extra fields, we could use `CodeDataMessage` as result type.
///
/// ```
/// async fn get_user(&self) -> ApiResult<User> {
///     let req = client.get("/api/path").await?;
///     let res: CodeDataMessage<User> = send!(req).await?;
///     // to access HTTP headers: res.get_header("name")
///     // to access extra fields: res.get_extra("other_field")
///     if res.is_success() {
///         Ok(res.data)
///     } else {
///         Err(ApiError::BusinessError(res.code, res.get_header().map(|v| v.to_string())))
///     }
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeDataMessage<T = Option<Value>> {
    /// `code` field
    pub code: i64,
    /// `data` field
    pub data: T,
    /// `message` or `msg` field
    #[serde(alias = "msg")]
    pub message: Option<String>,
    /// Hold all HTTP headers
    #[serde(rename = "__headers__", default)]
    headers: HashMap<String, String>,
    /// Hold unknown fields
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

impl<T> CodeDataMessage<T> {
    /// Check whether `code` is 0
    pub fn is_success(&self) -> bool {
        self.code == 0
    }

    /// Get any header
    /// - name: header name
    pub fn get_header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(|v| v.as_str())
    }

    /// Get any unknown field
    /// - name: field name
    pub fn get_extra<D>(&self, name: &str) -> Option<D>
    where
        D: DeserializeOwned,
    {
        self.extra
            .get(name)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Get `X-Request-ID` header
    pub fn get_request_id(&self) -> Option<&str> {
        self.get_header("X-Request-ID")
    }

    /// Get `X-Trace-ID` header
    pub fn get_trace_id(&self) -> Option<&str> {
        self.get_header("X-Trace-ID")
    }

    /// Get `X-Span-ID` header
    pub fn get_span_id(&self) -> Option<&str> {
        self.get_header("X-Span-ID")
    }
}

impl TryFrom<ResponseBody> for CodeDataMessage {
    type Error = ApiError;

    fn try_from(body: ResponseBody) -> Result<Self, Self::Error> {
        body.parse_json()
    }
}

impl JsonExtractor for CodeDataMessage {
    fn try_extract<T>(self) -> ApiResult<T>
    where
        T: DeserializeOwned,
    {
        match self.code {
            0 => {
                // Extract `data` field when `code` is 0
                match self.data {
                    Some(data) => {
                        serde_json::from_value(data).map_err(|_| ApiError::IllegalJson(Value::Null))
                    }
                    None => serde_json::from_value(Value::Null)
                        .map_err(|_| ApiError::IllegalJson(Value::Null)),
                }
            }
            code => {
                // Build error when `code` is not 0
                Err(ApiError::BusinessError(code, self.message))
            }
        }
    }
}

// impl Extractor for CodeDataMessage {
//     fn try_extract<T>(body: ResponseBody) -> ApiResult<T>
//     where
//         T: TryFrom<ResponseBody>,
//         T::Error: Into<ApiError>,
//     {
//         match body {
//             ResponseBody::Json(mut value) => {
//                 match value.get("code").and_then(|c| c.as_i64()) {
//                     // Extract `data` field when `code` is 0
//                     Some(0) => match value.get_mut("data") {
//                         Some(data) => {
//                             T::try_from(ResponseBody::Json(data.take())).map_err(|e| e.into())
//                         }
//                         None => T::try_from(ResponseBody::Json(Value::Null)).map_err(|e| e.into()),
//                     },
//                     // Build error when `code` is not 0
//                     Some(code) => {
//                         let message = value
//                             .get("message")
//                             .or_else(|| value.get("msg"))
//                             .and_then(|m| m.as_str())
//                             .map(|m| m.to_string());
//                         Err(ApiError::BusinessError(code, message))
//                     }
//                     // Failed to parse without `code` field
//                     None => Err(ApiError::IllegalJson(value)),
//                 }
//             }
//             ResponseBody::Xml(_xml) => Err(ApiError::Other),
//             ResponseBody::Text(_text) => Err(ApiError::Other),
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde_json::Value;

    use super::CodeDataMessage;

    #[derive(Debug, Deserialize)]
    #[allow(unused)]
    struct Payload {
        pub key: u32,
    }

    #[test]
    fn test_parse_null() {
        let v: Value = serde_json::from_str("null").unwrap();
        println!("v = {:?}", v);

        let v: Value = serde_json::from_value(Value::Null).unwrap();
        println!("v = {:?}", v);

        let v: Option<Value> = serde_json::from_str("null").unwrap();
        println!("v = {:?}", v);

        let v: Option<Value> = serde_json::from_value(Value::Null).unwrap();
        println!("v = {:?}", v);
    }

    #[test]
    fn test_cdm_data_miss_2_option_value() {
        let cdm: CodeDataMessage<Option<Value>> = serde_json::from_str(
            r#"
            {
                "code": 0
            }
            "#,
        )
        .unwrap();
        println!("test_cdm_data_miss_2_option_value = {:?}", cdm);
    }

    #[test]
    fn test_cdm_data_null_2_option_value() {
        let cdm: CodeDataMessage<Option<Value>> = serde_json::from_str(
            r#"
            {
                "code": 0,
                "data": null
            }
            "#,
        )
        .unwrap();
        println!("test_cdm_data_null_2_option_value = {:?}", cdm);
    }

    #[test]
    fn test_cdm_data_json_2_option_value() {
        let cdm: CodeDataMessage<Option<Value>> = serde_json::from_str(
            r#"
            {
                "code": 0,
                "data": {
                    "key": 1
                }
            }
            "#,
        )
        .unwrap();
        println!("test_cdm_data_json_2_option_value = {:?}", cdm);
    }

    #[test]
    #[should_panic]
    fn test_cdm_data_miss_2_value() {
        let cdm: CodeDataMessage<Value> = serde_json::from_str(
            r#"
            {
                "code": 0
            }
            "#,
        )
        .unwrap();
        println!("test_cdm_data_miss_2_value = {:?}", cdm);
    }

    #[test]
    fn test_cdm_data_null_2_value() {
        let cdm: CodeDataMessage<Value> = serde_json::from_str(
            r#"
            {
                "code": 0,
                "data": null
            }
            "#,
        )
        .unwrap();
        println!("test_cdm_data_null_2_value = {:?}", cdm);
    }

    #[test]
    fn test_cdm_data_json_2_value() {
        let cdm: CodeDataMessage<Value> = serde_json::from_str(
            r#"
            {
                "code": 0,
                "data": {
                    "key": 1
                }
            }
            "#,
        )
        .unwrap();
        println!("test_cdm_data_json_2_value = {:?}", cdm);
    }

    #[test]
    fn test_cdm_data_miss_2_option_payload() {
        let cdm: CodeDataMessage<Option<Payload>> = serde_json::from_str(
            r#"
            {
                "code": 0
            }
            "#,
        )
        .unwrap();
        println!("test_cdm_data_miss_2_option_payload = {:?}", cdm);
    }

    #[test]
    fn test_cdm_data_null_2_option_payload() {
        let cdm: CodeDataMessage<Option<Payload>> = serde_json::from_str(
            r#"
            {
                "code": 0,
                "data": null
            }
            "#,
        )
        .unwrap();
        println!("test_cdm_data_null_2_option_payload = {:?}", cdm);
    }

    #[test]
    fn test_cdm_data_json_2_option_payload() {
        let cdm: CodeDataMessage<Option<Payload>> = serde_json::from_str(
            r#"
            {
                "code": 0,
                "data": {
                    "key": 1
                }
            }
            "#,
        )
        .unwrap();
        println!("test_cdm_data_json_2_option_payload = {:?}", cdm);
    }

    #[test]
    #[should_panic]
    fn test_cdm_data_miss_2_payload() {
        let cdm: CodeDataMessage<Payload> = serde_json::from_str(
            r#"
            {
                "code": 0
            }
            "#,
        )
        .unwrap();
        println!("test_cdm_data_miss_2_payload = {:?}", cdm);
    }

    #[test]
    #[should_panic]
    fn test_cdm_data_null_2_payload() {
        let cdm: CodeDataMessage<Payload> = serde_json::from_str(
            r#"
            {
                "code": 0,
                "data": null
            }
            "#,
        )
        .unwrap();
        println!("test_cdm_data_null_2_payload = {:?}", cdm);
    }

    #[test]
    fn test_cdm_data_json_2_payload() {
        let cdm: CodeDataMessage<Payload> = serde_json::from_str(
            r#"
            {
                "code": 0,
                "data": {
                    "key": 1
                }
            }
            "#,
        )
        .unwrap();
        println!("test_cdm_data_json_2_payload = {:?}", cdm);
    }

    #[test]
    fn test_cdm_extra() {
        let cdm: CodeDataMessage = serde_json::from_str(
            r#"
            {
                "code": 0,
                "num": 1,
                "text": "string"
            }
            "#,
        )
        .unwrap();
        println!("{:?}", cdm);
        println!("extra.num = {:?}", cdm.get_extra::<u32>("num"));
        println!("extra.text = {:?}", cdm.get_extra::<String>("text"));
    }
}
