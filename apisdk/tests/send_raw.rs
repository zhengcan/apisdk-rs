use apisdk::{send_raw, ApiResult};
use reqwest::Response;

use crate::common::{init_logger, start_server, TheApi};

mod common;

impl TheApi {
    async fn touch_200(&self) -> ApiResult<Response> {
        let req = self.get("/path/json").await?;
        send_raw!(req).await
    }

    async fn touch_405(&self) -> ApiResult<Response> {
        let req = self.get("/not-found").await?;
        send_raw!(req).await
    }
}

#[tokio::test]
async fn test_send_raw_200() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.touch_200().await?;
    log::debug!("res = {:?}", res);
    assert_eq!(200, res.status().as_u16());

    Ok(())
}

#[tokio::test]
async fn test_send_raw_405() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.touch_405().await?;
    log::debug!("res = {:?}", res);
    assert_eq!(405, res.status().as_u16());

    Ok(())
}
