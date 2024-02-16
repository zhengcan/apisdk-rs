use apisdk::{send, ApiResult, UrlOps};
use url::Url;

use crate::common::{init_logger, TheApi};

mod common;

impl TheApi {
    async fn touch(&self) -> ApiResult<()> {
        let req = self.get("/path/json").await?;
        send!(req).await
    }
}

#[tokio::test]
async fn test_rewrite() -> ApiResult<()> {
    init_logger();

    let api = TheApi::builder()
        .with_rewriter(|url: Url| Ok(url.merge_path("/more/")))
        .build();
    println!("api = {:?}", api);

    let result = api.touch().await;
    println!("result = {:?}", result);

    Ok(())
}
