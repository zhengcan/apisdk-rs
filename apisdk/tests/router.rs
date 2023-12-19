use std::sync::atomic::AtomicBool;

use apisdk::{
    send, ApiEndpoint, ApiResult, ApiRouter, ApiRouters, CodeDataMessage, OriginalEndpoint,
    RouteError,
};
use async_trait::async_trait;
use common::Payload;

use crate::common::{init_logger, start_server, TheApi, PORT};

mod common;

impl TheApi {
    async fn touch(&self) -> ApiResult<Payload> {
        let req = self.get("/path/json").await?;
        send!(req, CodeDataMessage).await
    }
}

#[tokio::test]
async fn test_reserve_host() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder()
        .with_router(ApiRouters::fixed(("127.0.0.1", PORT)))
        .build();

    let res = api.touch().await?;
    log::debug!("res = {:?}", res);
    let host = res
        .headers
        .get("host")
        .map(|v| v.to_string())
        .unwrap_or_default();
    assert_eq!("localhost", host);

    Ok(())
}

#[tokio::test]
async fn test_route_error() -> ApiResult<()> {
    init_logger();
    start_server().await;

    #[derive(Debug)]
    struct MyRouter {
        flag: AtomicBool,
    }

    #[async_trait]
    impl ApiRouter for MyRouter {
        async fn next_endpoint(&self) -> Result<Box<dyn ApiEndpoint>, RouteError> {
            let flag = self
                .flag
                .fetch_xor(true, std::sync::atomic::Ordering::AcqRel);
            if flag {
                Ok(Box::new(OriginalEndpoint::default()))
            } else {
                Err(RouteError::ServiceDiscovery(anyhow::format_err!(
                    "Some error"
                )))
            }
        }
    }

    let api = TheApi::builder()
        .with_router(MyRouter {
            flag: AtomicBool::new(false),
        })
        .build();

    let res = api.touch().await;
    log::debug!("res = {:?}", res);
    assert!(res.is_err());

    let res = api.touch().await;
    log::debug!("res = {:?}", res);
    assert!(res.is_ok());

    Ok(())
}
