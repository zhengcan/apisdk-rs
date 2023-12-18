use apisdk::{api_method, send, ApiResult};
use serde_json::Value;

use crate::common::{init_logger, start_server, TheApi};

mod common;

impl TheApi {
    #[api_method(log = true)]
    async fn touch(&self) -> ApiResult<Value> {
        let req = self.get("/path/json").await?;
        send!(req, Value).await
    }
}

#[tokio::test]
async fn test_api_method() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.touch().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}
