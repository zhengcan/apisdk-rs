use apisdk::{
    send_multipart, ApiResult, CodeDataMessage, DynamicForm, MultipartForm, MultipartFormOps,
};
use serde_json::Value;

use crate::common::{init_logger, start_server, TheApi};

mod common;

impl TheApi {
    async fn multipart_via_dynamic_form(&self) -> ApiResult<Value> {
        let req = self.post("/path/multipart").await?;
        let form = DynamicForm::new()
            .text("key1", 1.to_string())
            .text("key2", 2.to_string())
            .text("key3", 3.to_string());
        send_multipart!(req, form, CodeDataMessage).await
    }

    async fn multipart_via_multipart_form(&self) -> ApiResult<Value> {
        let req = self.post("/path/multipart").await?;
        let form = MultipartForm::new()
            .text("key1", 1.to_string())
            .text("key2", 2.to_string())
            .text("key3", 3.to_string());
        send_multipart!(req, form, CodeDataMessage).await
    }
}

#[tokio::test]
async fn test_send_multipart_via_dynamic_form() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.multipart_via_dynamic_form().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_send_multipart_via_multipart_form() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.multipart_via_multipart_form().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}
