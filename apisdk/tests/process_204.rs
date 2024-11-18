use apisdk::serde_json::json;
use apisdk::{http_api, send_json, ApiResult};

use crate::common::init_logger;

mod common;

#[http_api("http://localhost:8000/")]
pub struct SampleAPI;

impl SampleAPI {
    pub async fn reply_204(&self) -> ApiResult<()> {
        let payload = json!({"name": "test"});
        let req = self.post("/test").await?;
        send_json!(req, payload, ()).await
    }
}

#[tokio::test]
async fn test_send_form_via_hashmap() -> ApiResult<()> {
    init_logger();

    let api = SampleAPI::default();

    let res = api.reply_204().await?;

    Ok(())
}
