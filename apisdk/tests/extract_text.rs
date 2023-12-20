use apisdk::{send, ApiResult};

use crate::common::{init_logger, start_server, TheApi};

mod common;

impl TheApi {
    async fn get_string(&self) -> ApiResult<String> {
        let req = self.get("/path/text").await?;
        send!(req, Text).await
    }
}

#[tokio::test]
async fn test_extract_text_string() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().with_log(true).build();

    let res = api.get_string().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}
