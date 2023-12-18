use apisdk::{send, ApiResult, CodeDataMessage};
use serde_json::Value;

use crate::common::{init_logger, start_server, TheApi};

mod common;

impl TheApi {
    async fn get_as_value(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req).await
    }

    async fn get_as_cdm(&self) -> ApiResult<CodeDataMessage> {
        let req = self.get("/path/json").await?;
        send!(req).await
    }

    async fn get_and_extract_value(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req, Value).await
    }

    async fn get_and_extract_cdm(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req, CodeDataMessage).await
    }
}

#[tokio::test]
async fn test_send_get_as_value() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.get_as_value().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_send_get_as_cdm() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.get_as_cdm().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_send_get_and_extract_value() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.get_and_extract_value().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_send_get_and_extract_cdm() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.get_and_extract_cdm().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}
