use std::sync::atomic::AtomicBool;

use apisdk::{
    send, ApiEndpoint, ApiResult, ApiRouter, ApiRouters, CodeDataMessage, OriginalEndpoint,
    RouteError, UrlRewrite,
};
use apisdk_macros::http_api;
use async_trait::async_trait;
use common::Payload;
use url::Url;

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
    impl UrlRewrite for MyRouter {
        async fn rewrite(&self, url: Url) -> Url {
            url
        }
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

#[tokio::test]
async fn test_api_with_member() -> ApiResult<()> {
    init_logger();

    #[http_api("http://host/path")]
    #[derive(Debug)]
    struct NewApi {
        value: u32,
    }

    let mut api = NewApi::builder()
        .with_router(ApiRouters::fixed(("127.0.0.1", 80)))
        .build();
    api.value = 666;
    println!("api = {:?}", api);

    let api2 = api.with_endpoint(("127.0.0.1", 80));
    println!("api2 = {:?}", api2);

    assert_eq!(api.value, api2.value);

    Ok(())
}
