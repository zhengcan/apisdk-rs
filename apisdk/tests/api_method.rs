use apisdk::{api_method, send, ApiResult};
use serde_json::Value;

use crate::common::{init_logger, start_server, TheApi};

mod common;

impl TheApi {
    #[api_method(log = false)]
    async fn bool_to_off(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req, Value).await
    }

    #[api_method(log = true)]
    async fn bool_to_on(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req, Value).await
    }

    #[api_method(log = "off")]
    async fn str_to_off(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req, Value).await
    }

    #[api_method(log = "info")]
    async fn str_to_info(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req, Value).await
    }

    #[api_method(log = "error")]
    async fn str_to_error(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req, Value).await
    }

    #[api_method(log = "unknown")]
    async fn str_to_unknown(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req, Value).await
    }
}

#[tokio::test]
async fn test_api_method_bool_to_off() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.bool_to_off().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_api_method_bool_to_on() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.bool_to_on().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_api_method_str_to_off() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.str_to_off().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_api_method_str_to_info() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.str_to_info().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_api_method_str_to_error() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.str_to_error().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_api_method_str_to_unknown() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.str_to_unknown().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}
