use std::collections::HashMap;

use apisdk::{send_form, ApiResult, CodeDataMessage, DynamicForm, MultipartForm, MultipartFormOps};
use serde_json::{json, Value};

use crate::common::{init_logger, start_server, TheApi};

mod common;

impl TheApi {
    async fn form_via_hashmap(&self) -> ApiResult<Value> {
        let req = self.post("/path/form").await?;
        let form = HashMap::from([("key1", 1), ("key2", 2), ("key3", 3)]);
        send_form!(req, form).await
    }

    async fn form_via_hashmap2(&self) -> ApiResult<Value> {
        let req = self.post("/path/form").await?;
        let mut form = HashMap::new();
        form.insert("key1", "value1");
        form.insert("key2", "value2");
        form.insert("key3", "value3");
        send_form!(req, form, CodeDataMessage).await
    }

    async fn form_via_json(&self) -> ApiResult<Value> {
        let req = self.post("/path/form").await?;
        let form = json!({
            "key1": 1,
            "key2": 2,
            "key3": 3,
        });
        send_form!(req, form, CodeDataMessage).await
    }

    async fn form_via_dynamic_form(&self) -> ApiResult<Value> {
        let req = self.post("/path/form").await?;
        let form = DynamicForm::new()
            .text("key1", 1.to_string())
            .text("key2", 2.to_string())
            .text("key3", 3.to_string());
        send_form!(req, form, CodeDataMessage).await
    }

    async fn form_via_multipart_form(&self) -> ApiResult<Value> {
        let req = self.post("/path/form").await?;
        let form = MultipartForm::new()
            .text("key1", 1.to_string())
            .text("key2", 2.to_string())
            .text("key3", 3.to_string());
        send_form!(req, form, CodeDataMessage).await
    }
}

#[tokio::test]
async fn test_send_form_via_hashmap() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.form_via_hashmap().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_send_form_via_hashmap2() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.form_via_hashmap2().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_send_form_via_json() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.form_via_json().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_send_form_via_dynamic_form() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.form_via_dynamic_form().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
#[should_panic]
async fn test_send_form_via_multipart_form() {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.form_via_multipart_form().await.unwrap();
    log::debug!("res = {:?}", res);
}
