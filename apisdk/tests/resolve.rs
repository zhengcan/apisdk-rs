#![cfg(not(target_arch = "wasm32"))]

use std::net::IpAddr;

use apisdk::{send, ApiResult, DnsResolver, SocketAddrs};
use apisdk_macros::http_api;
use async_trait::async_trait;

use crate::common::{init_logger, start_server, TheApi};

mod common;

impl TheApi {
    async fn touch(&self) -> ApiResult<()> {
        let req = self.get("/path/json").await?;
        send!(req).await
    }
}

#[tokio::test]
async fn test_resolver_simple_with_port() -> ApiResult<()> {
    init_logger();

    let api = TheApi::builder()
        .with_resolver(([127, 0, 0, 66], 80))
        .build();
    println!("api = {:?}", api);

    let result = api.touch().await;
    println!("result = {:?}", result);

    Ok(())
}

#[tokio::test]
async fn test_resolver_simple_without_port() -> ApiResult<()> {
    init_logger();

    let api = TheApi::builder()
        .with_resolver(IpAddr::from([127, 0, 0, 66]))
        .build();
    println!("api = {:?}", api);

    let result = api.touch().await;
    println!("result = {:?}", result);

    Ok(())
}

#[tokio::test]
async fn test_resolver_full() -> ApiResult<()> {
    init_logger();

    struct FullResolver;

    #[async_trait]
    impl DnsResolver for FullResolver {
        fn get_scheme(&self) -> Option<&str> {
            Some("https")
        }

        fn get_port(&self) -> Option<u16> {
            Some(9966)
        }

        async fn resolve(&self, _: &str) -> Option<SocketAddrs> {
            Some(SocketAddrs::from(([127, 0, 0, 66], 80)))
        }
    }

    let api = TheApi::builder().with_resolver(FullResolver).build();
    println!("api = {:?}", api);

    let result = api.touch().await;
    println!("result = {:?}", result);

    Ok(())
}

#[tokio::test]
async fn test_resolver_keep_hostname() -> ApiResult<()> {
    init_logger();
    start_server().await;

    #[http_api("http://external/v1")]
    #[derive(Debug)]
    struct ExternalApi;

    impl ExternalApi {
        async fn touch(&self) -> ApiResult<()> {
            let req = self.get("/path/json").await?;
            send!(req).await
        }
    }

    let api = ExternalApi::builder()
        .with_resolver(([127, 0, 0, 1], 3030))
        .build();
    println!("api = {:?}", api);

    let result = api.touch().await;
    println!("result = {:?}", result);

    Ok(())
}
