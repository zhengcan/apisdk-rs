use std::net::SocketAddr;

use apisdk::{send, ApiResult, DnsResolver};
use async_trait::async_trait;

use crate::common::{init_logger, TheApi};

mod common;

impl TheApi {
    async fn touch(&self) -> ApiResult<()> {
        let req = self.get("/path/json").await?;
        send!(req).await
    }
}

#[tokio::test]
async fn test_with_resolver_simple() -> ApiResult<()> {
    init_logger();

    let api = TheApi::builder()
        .with_resolver(([127, 0, 0, 66], 80))
        .build();
    let result = api.touch().await;
    println!("result = {:?}", result);

    Ok(())
}

#[tokio::test]
async fn test_with_resolver_full() -> ApiResult<()> {
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

        async fn resolve(&self, _: &str) -> Option<SocketAddr> {
            Some(SocketAddr::from(([127, 0, 0, 66], 80)))
        }
    }

    let api = TheApi::builder().with_resolver(FullResolver).build();
    let result = api.touch().await;
    println!("result = {:?}", result);

    Ok(())
}

// #[tokio::test]
// async fn test_reserve_host() -> ApiResult<()> {
//     init_logger();
//     start_server().await;

//     let api = TheApi::builder()
//         .with_router(ApiRouters::fixed(("127.0.0.1", PORT)))
//         .build();

//     let res = api.touch().await?;
//     log::debug!("res = {:?}", res);
//     let host = res
//         .headers
//         .get("host")
//         .map(|v| v.to_string())
//         .unwrap_or_default();
//     assert_eq!("localhost", host);

//     Ok(())
// }

// #[tokio::test]
// async fn test_route_error() -> ApiResult<()> {
//     init_logger();
//     start_server().await;

//     #[derive(Debug)]
//     struct MyRouter {
//         flag: AtomicBool,
//     }

//     #[async_trait]
//     impl UrlRewrite for MyRouter {
//         async fn rewrite(&self, url: Url) -> Url {
//             url
//         }
//     }

//     #[async_trait]
//     impl ApiRouter for MyRouter {
//         async fn next_endpoint(&self) -> Result<Box<dyn ApiEndpoint>, RouteError> {
//             let flag = self
//                 .flag
//                 .fetch_xor(true, std::sync::atomic::Ordering::AcqRel);
//             if flag {
//                 Ok(Box::new(OriginalEndpoint::default()))
//             } else {
//                 Err(RouteError::ServiceDiscovery(anyhow::format_err!(
//                     "Some error"
//                 )))
//             }
//         }
//     }

//     let api = TheApi::builder()
//         .with_router(MyRouter {
//             flag: AtomicBool::new(false),
//         })
//         .build();

//     let res = api.touch().await;
//     log::debug!("res = {:?}", res);
//     assert!(res.is_err());

//     let res = api.touch().await;
//     log::debug!("res = {:?}", res);
//     assert!(res.is_ok());

//     Ok(())
// }

// #[tokio::test]
// async fn test_api_with_member() -> ApiResult<()> {
//     init_logger();

//     #[http_api("http://host/path")]
//     #[derive(Debug)]
//     struct NewApi {
//         value: u32,
//     }

//     let mut api = NewApi::builder()
//         .with_router(ApiRouters::fixed(("127.0.0.1", 80)))
//         .build();
//     api.value = 666;
//     println!("api = {:?}", api);

//     let api2 = api.with_endpoint(("127.0.0.1", 80));
//     println!("api2 = {:?}", api2);

//     assert_eq!(api.value, api2.value);

//     Ok(())
// }
