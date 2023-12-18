use std::collections::HashMap;

use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;

use crate::{ApiError, ApiResult};

/// This trait is used to extract result from response.
///
/// # Usage
///
/// ```
/// let req = client.get("/api/path").await?;
/// let res = send!(req, TypeOfJsonExtractor).await?;
/// ```
///
/// # Examples
///
/// ### Check return code
///
/// ```
/// pub struct CheckReturnCode;
///
/// impl JsonExtractor for CheckReturnCode {
///     fn try_extract<T>(value: Value) -> ApiResult<T> {
///         match value.get("ret_code").and_then(|c| c.as_i64()) {
///             Some(0) => serde_json::from_value(value).map_err(|e| e.into()),
///             Some(c) => Err(ApiError::BusinessError(c, Some("Invalid ret_code"))),
///             None => Err(ApiError::BusinessError(c, Some("No ret_code"))),
///         }
///     }
/// }
/// ```
///
/// ### Extract single field
///
/// ```
/// pub struct ExtractData;
///
/// impl JsonExtractor for ExtractData {
///     fn try_extract<T>(value: Value) -> ApiResult<T> {
///         let data = value.get("data").unwrap_or(Value::Null);
///         serde_json::from_value(data).map_err(|e| e.into())
///     }
/// }
/// ```
///
/// # Built-in Extractors
///
/// - serde_json::Value
///     - treat whole payload as output
/// - apisdk::WholePayload
///     - an alias of serde_json::Value
/// - apisdk::CodeDataMessage
///     - parse `{code, data, message}` payload, and return `data` field
pub trait JsonExtractor {
    /// The extractor needs response HTTP headers or not.
    fn require_headers() -> bool {
        false
    }

    /// Try to extract result from response.
    ///
    /// The HTTP headers will be inject as `__headers__` field if possible.
    /// - value: the response payload
    fn try_extract<T>(value: Value) -> ApiResult<T>
    where
        T: DeserializeOwned;
}

impl JsonExtractor for Value {
    fn try_extract<T>(value: Value) -> ApiResult<T>
    where
        T: DeserializeOwned,
    {
        serde_json::from_value(value).map_err(ApiError::DecodeJson)
    }
}

/// This extractor will treat whole payload as result
pub type WholePayload = Value;

/// This struct is used to parse `{code, data, message}` payload.
///
/// When it's used as `JsonExtractor`, it will extract `data` from payload.
///
/// # Examples
///
/// ### As JsonExtractor
///
/// To be used as `JsonExtractor`, `CodeDataMessage` will check `code` field of response payload, and ensure it must be `0`.
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
#[derive(Debug, Deserialize)]
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

impl JsonExtractor for CodeDataMessage {
    fn try_extract<T>(value: Value) -> ApiResult<T>
    where
        T: DeserializeOwned,
    {
        let mut value = value;
        match value.get("code").and_then(|c| c.as_i64()) {
            // Extract `data` field when `code` is 0
            Some(0) => match value.get_mut("data") {
                Some(data) => serde_json::from_value(data.take()).map_err(ApiError::DecodeJson),
                None => serde_json::from_value(Value::Null).map_err(ApiError::DecodeJson),
            },
            // Build error when `code` is not 0
            Some(code) => {
                let message = value
                    .get("message")
                    .or_else(|| value.get("msg"))
                    .and_then(|m| m.as_str())
                    .map(|m| m.to_string());
                Err(ApiError::BusinessError(code, message))
            }
            // Failed to parse without `code` field
            None => Err(ApiError::InvalidJson(value)),
        }
    }
}

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
