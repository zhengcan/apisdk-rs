use apisdk::{send, ApiError, ApiResult, CodeDataMessage, JsonExtractor};
use serde::Deserialize;
use serde_json::Value;

use crate::common::{init_logger, start_server, TheApi};

mod common;

#[derive(Debug, Deserialize)]
struct HasHeaders(Value);

impl JsonExtractor for HasHeaders {
    fn require_headers() -> bool {
        true
    }

    fn try_extract<T>(mut self) -> ApiResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        assert!(self.0.get("__headers__").is_some());
        match self.0.get("code").and_then(|c| c.as_i64()) {
            Some(0) => match self.0.get_mut("data") {
                Some(data) => serde_json::from_value(data.take()).map_err(ApiError::DecodeJson),
                None => serde_json::from_value(Value::Null).map_err(ApiError::DecodeJson),
            },
            Some(c) => Err(ApiError::ServiceError(
                c,
                Some("Invalid ret_code".to_string()),
            )),
            None => Err(ApiError::ServiceError(-1, Some("No ret_code".to_string()))),
        }
    }
}

#[derive(Debug, Deserialize)]
struct NoHeaders(Value);

impl JsonExtractor for NoHeaders {
    fn require_headers() -> bool {
        false
    }

    fn try_extract<T>(mut self) -> ApiResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        assert!(self.0.get("__headers__").is_none());
        match self.0.get_mut("data") {
            Some(data) => serde_json::from_value(data.take()).map_err(ApiError::DecodeJson),
            None => serde_json::from_value(Value::Null).map_err(ApiError::DecodeJson),
        }
    }
}

impl TheApi {
    async fn get_json_2_string(&self) -> ApiResult<String> {
        let req = self.get("/path/json").await?;
        send!(req, Json).await
    }

    async fn get_json_2_value(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req, Json).await
    }

    async fn extract_value_2_value(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req, Value).await
    }

    async fn extract_cdm_2_value(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req, CodeDataMessage).await
    }

    async fn extract_json_cdm_2_value(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req, Json<CodeDataMessage>).await
    }

    async fn extract_custom_has_headers(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req, HasHeaders).await
    }

    async fn extract_custom_no_headers(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req, NoHeaders).await
    }
}

#[tokio::test]
async fn test_extract_json_string() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.get_json_2_string().await?;
    log::debug!("res = {:?}", res);
    assert!(res.contains("code"));

    Ok(())
}

#[tokio::test]
async fn test_extract_json_value() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.get_json_2_value().await?;
    log::debug!("res = {:?}", res);
    assert!(res.get("code").is_some());

    Ok(())
}

#[tokio::test]
async fn test_extract_json_value_2_value() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.extract_value_2_value().await?;
    log::debug!("res = {:?}", res);
    assert!(res.get("code").is_some());

    Ok(())
}

#[tokio::test]
async fn test_extract_json_cdm_2_value() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.extract_cdm_2_value().await?;
    log::debug!("res = {:?}", res);
    assert!(res.get("path").is_some());

    Ok(())
}

#[tokio::test]
async fn test_extract_json_json_cdm_2_value() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.extract_json_cdm_2_value().await?;
    log::debug!("res = {:?}", res);
    assert!(res.get("path").is_some());

    Ok(())
}

#[tokio::test]
async fn test_extract_custom() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.extract_custom_has_headers().await?;
    log::debug!("res = {:?}", res);

    let res = api.extract_custom_no_headers().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}
