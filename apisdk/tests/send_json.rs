use apisdk::{send_json, ApiResult, CodeDataMessage};
use serde_json::{json, Value};

use crate::common::{init_logger, start_server, TheApi};

mod common;

impl TheApi {
    async fn post_as_value(&self) -> ApiResult<Value> {
        let req = self.post("/path/json").await?;
        let payload = json!({
            "num": 1,
            "text": "string",
        });
        send_json!(req, payload).await
    }

    async fn post_and_extract_cdm(&self) -> ApiResult<Value> {
        let req = self.post("/path/json").await?;
        let payload = json!({
            "num": 1,
            "text": "string",
        });
        send_json!(req, payload, CodeDataMessage).await
    }
}

#[tokio::test]
async fn test_send_post_as_value() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.post_as_value().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_send_post_and_extract_cdm() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.post_and_extract_cdm().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}
