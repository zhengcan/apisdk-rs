use apisdk::{
    send, AccessTokenAuth, ApiAuthenticator, ApiResult, Carrier, CodeDataMessage, HashedTokenAuth,
    TokenGenerator, WithCarrier,
};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine};
use reqwest::{header::AUTHORIZATION, Request};

use crate::common::{init_logger, start_server, Payload, TheApi};

mod common;

impl TheApi {
    async fn touch(&self) -> ApiResult<Payload> {
        let req = self.get("/path/json").await?;
        send!(req, CodeDataMessage).await
    }
}

#[tokio::test]
async fn test_access_token_auth_fixed() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder()
        .with_authenticator(AccessTokenAuth::new("fixed"))
        .build();

    let res = api.touch().await?;
    log::debug!("res = {:?}", res);
    let auth = res.headers.get("authorization").unwrap();
    assert_eq!("Bearer fixed", auth);

    Ok(())
}

#[tokio::test]
async fn test_access_token_auth_dynamic() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder()
        .with_authenticator(AccessTokenAuth::new_dynamic(|| Ok("dynamic")))
        .build();

    let res = api.touch().await?;
    log::debug!("res = {:?}", res);
    let auth = res.headers.get(AUTHORIZATION.as_str()).unwrap();
    assert_eq!("Bearer dynamic", auth);

    Ok(())
}

#[tokio::test]
async fn test_access_token_auth_in_header() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder()
        .with_authenticator(AccessTokenAuth::new("fixed").with_header_name("x-auth"))
        .build();

    let res = api.touch().await?;
    log::debug!("res = {:?}", res);
    let auth = res.headers.get("x-auth").unwrap();
    assert_eq!("fixed", auth);

    Ok(())
}

#[tokio::test]
async fn test_access_token_auth_schemeless() -> ApiResult<()> {
    init_logger();
    start_server().await;

    struct Schemeless {}

    #[async_trait]
    impl TokenGenerator for Schemeless {
        async fn generate_token(
            &self,
            _req: &Request,
        ) -> Result<String, reqwest_middleware::Error> {
            Ok("token".to_string())
        }
    }
    #[async_trait]
    impl ApiAuthenticator for Schemeless {
        fn get_carrier(&self) -> &Carrier {
            &Carrier::SchemalessAuth
        }
    }

    let api = TheApi::builder().with_authenticator(Schemeless {}).build();

    let res = api.touch().await?;
    log::debug!("res = {:?}", res);
    let auth = res.headers.get(AUTHORIZATION.as_str()).unwrap();
    assert_eq!("token", auth);

    Ok(())
}

#[tokio::test]
async fn test_access_token_auth_in_query() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder()
        .with_authenticator(AccessTokenAuth::new("fixed").with_query_param("x-auth"))
        .build();

    let res = api.touch().await?;
    log::debug!("res = {:?}", res);
    let auth = res.query.get("x-auth").unwrap();
    assert_eq!("fixed", auth);

    Ok(())
}

#[tokio::test]
async fn test_hashed_token_auth() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder()
        .with_authenticator(HashedTokenAuth::new("app_id", "app_secret"))
        .build();

    let res = api.touch().await?;
    log::debug!("res = {:?}", res);
    let auth = res.headers.get("authorization").unwrap();
    assert!(auth.starts_with("Bearer "));
    let token = auth.trim_start_matches("Bearer ");
    let decoded = general_purpose::STANDARD.decode(token).unwrap();
    let decoded = String::from_utf8(decoded).unwrap();
    log::debug!("decoded = {}", decoded);
    assert!(decoded.starts_with("app_id,"));

    Ok(())
}
