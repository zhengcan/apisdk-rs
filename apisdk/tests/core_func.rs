use apisdk::{ApiResult, ApiRouters};

use crate::common::{init_logger, TheApi};

mod common;

impl TheApi {
    async fn core_build_url(&self, path: impl AsRef<str>) -> ApiResult<()> {
        let url = self.core.build_url(path).await?;
        log::info!("url = {:?}", url);
        Ok(())
    }
}

#[tokio::test]
async fn test_via_api() -> ApiResult<()> {
    init_logger();

    let api = TheApi::default();

    let api2 = api.with_endpoint(("127.0.0.1", 80));
    log::info!("api2 = {:?}", api2);

    let api3 = api.with_router(ApiRouters::random(&[
        ("127.0.0.1", 80).into(),
        ("127.0.0.1", 80).into(),
        ("127.0.0.1", 80).into(),
    ]));
    log::info!("api3 = {:?}", api3);

    Ok(())
}

#[tokio::test]
async fn test_via_core() -> ApiResult<()> {
    init_logger();

    let api = TheApi::default();

    api.core_build_url("path").await?;

    Ok(())
}
