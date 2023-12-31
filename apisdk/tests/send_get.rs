use apisdk::{send, ApiError, ApiResult, CodeDataMessage, ResponseBody};
use serde::Deserialize;
use serde_json::Value;

use crate::common::{init_logger, start_server, TheApi};

mod common;

#[derive(Debug)]
#[allow(unused)]
struct CustomCodeData {
    code: i64,
    data: Value,
}

impl TryFrom<ResponseBody> for CustomCodeData {
    type Error = ApiError;

    fn try_from(body: ResponseBody) -> Result<Self, Self::Error> {
        let scd: SimpleCodeData = body.parse_json()?;
        Ok(Self {
            code: scd.code,
            data: scd.data,
        })
    }
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct SimpleCodeData {
    code: i64,
    data: Value,
}

impl TheApi {
    async fn get_as_value(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req).await
    }

    async fn get_as_cdm(&self) -> ApiResult<CodeDataMessage> {
        let req = self.get("/path/json").await?;
        send!(req).await
    }

    async fn get_as_ccd(&self) -> ApiResult<CustomCodeData> {
        let req = self.get("/path/json").await?;
        send!(req, Body).await
    }

    async fn get_as_scd(&self) -> ApiResult<SimpleCodeData> {
        let req = self.get("/path/json").await?;
        send!(req).await
    }

    async fn get_as_unit(&self) -> ApiResult<()> {
        let req = self.get("/path/json").await?;
        send!(req, ()).await
    }

    async fn get_and_extract_value(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req, Value).await
    }

    async fn get_and_extract_text(&self) -> ApiResult<String> {
        let req = self.get("/path/json").await?;
        send!(req, String).await
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
async fn test_send_get_as_ccd() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.get_as_ccd().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_send_get_as_scd() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.get_as_scd().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_send_get_as_unit() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.get_as_unit().await?;
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
async fn test_send_get_and_extract_text() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.get_and_extract_text().await?;
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
