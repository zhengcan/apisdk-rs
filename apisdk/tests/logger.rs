use apisdk::{send, ApiResult, CodeDataMessage, LogConfig};

use crate::common::{init_logger, start_server, Payload, TheApi};

mod common;

impl TheApi {
    async fn none(&self) -> ApiResult<Payload> {
        let req = self.get("/path/json").await?;
        send!(req, CodeDataMessage).await
    }

    async fn off(&self) -> ApiResult<Payload> {
        let req = self.get("/path/json").await?;
        let req = req.with_extension(LogConfig::off());
        send!(req, CodeDataMessage).await
    }

    async fn def(&self) -> ApiResult<Payload> {
        let req = self.get("/path/json").await?;
        let req = req.with_extension(LogConfig::default());
        send!(req, CodeDataMessage).await
    }

    async fn info(&self) -> ApiResult<Payload> {
        let req = self.get("/path/json").await?;
        let req = req.with_extension(LogConfig::new("info"));
        send!(req, CodeDataMessage).await
    }

    async fn error(&self) -> ApiResult<Payload> {
        let req = self.get("/path/json").await?;
        let req = req.with_extension(LogConfig::new("error"));
        send!(req, CodeDataMessage).await
    }
}

#[tokio::test]
async fn test_log_as_defautl_none() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::default();

    let res = api.none().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_log_as_none() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.none().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_log_as_off() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.off().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_log_as_default() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.def().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_log_as_info() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.info().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_log_as_error() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.error().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}
