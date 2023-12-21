use apisdk::{send, ApiError, ApiResult, CodeDataMessage, MockServer, ResponseBody};
use serde::Deserialize;
use serde_json::json;

use crate::common::{init_logger, start_server, TheApi};

mod common;

#[derive(Debug, Deserialize)]
pub struct MockPayload {
    #[serde(default)]
    pub mock: bool,
    #[serde(default)]
    pub message: Option<String>,
}

impl TryFrom<ResponseBody> for MockPayload {
    type Error = ApiError;

    fn try_from(body: ResponseBody) -> Result<Self, Self::Error> {
        body.parse_json()
    }
}

impl TheApi {
    async fn touch(&self) -> ApiResult<MockPayload> {
        let req = self.get("/path/json").await?;
        send!(req, CodeDataMessage).await
    }

    async fn touch_mock(&self) -> ApiResult<MockPayload> {
        let req = self.get("/path/json").await?;
        let req = req.with_extension(MockServer::new(|_| {
            Ok(ResponseBody::Json(json!({
                "code": 0,
                "data": {
                    "mock": true
                }
            })))
        }));
        send!(req, CodeDataMessage).await
    }
}

#[tokio::test]
async fn test_mock_single() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.touch_mock().await?;
    log::debug!("res = {:?}", res);
    assert!(res.mock);

    Ok(())
}

#[tokio::test]
async fn test_mock_all() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder()
        .with_initialiser(MockServer::new(|_| {
            Ok(ResponseBody::Json(json!({
                "code": 0,
                "data": {
                    "mock": true
                }
            })))
        }))
        .build();

    let res = api.touch().await?;
    log::debug!("res = {:?}", res);
    assert!(res.mock);

    Ok(())
}

#[tokio::test]
async fn test_mock_error() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder()
        .with_initialiser(MockServer::new(|_| Err(anyhow::format_err!("any error"))))
        .build();

    let res = api.touch().await;
    log::debug!("res = {:?}", res);
    assert!(res.is_err());

    Ok(())
}
