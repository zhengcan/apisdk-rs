use apisdk::{send, ApiResult, CodeDataMessage};
use serde_json::Value;

use crate::common::{init_logger, start_server, TheApi};

mod common;

impl TheApi {
    async fn touch_as_json(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req).await
    }

    async fn touch_as_string(&self) -> ApiResult<String> {
        let req = self.get("/path/json").await?;
        send!(req).await
    }

    async fn touch_as_cdm(&self) -> ApiResult<CodeDataMessage> {
        let req = self.get("/path/json").await?;
        send!(req).await
    }

    async fn touch_unit(&self) -> ApiResult<()> {
        let req = self.get("/path/json").await?;
        send!(req, ()).await
    }
}

#[tokio::test]
async fn test_touch_as_json() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.touch_as_json().await?;
    log::debug!("res = {:?}", res);
    assert!(res.get("code").is_some());
    assert!(res.get("__headers__").is_some());

    Ok(())
}

#[tokio::test]
async fn test_touch_as_string() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.touch_as_string().await?;
    log::debug!("res = {:?}", res);
    assert!(res.contains("code"));
    assert!(!res.contains("__headers__"));

    Ok(())
}

#[tokio::test]
async fn test_touch_as_cdm() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.touch_as_cdm().await?;
    log::debug!("res = {:?}", res);
    assert!(res.get_header("content-length").is_some());

    Ok(())
}

#[tokio::test]
async fn test_touch_unit() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.touch_unit().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}
