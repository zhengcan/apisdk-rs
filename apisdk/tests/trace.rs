use apisdk::{send, ApiResult, CodeDataMessage, RequestId, TraceId};
use serde::Deserialize;

use crate::common::{init_logger, start_server, Payload, TheApi};

mod common;

#[derive(Debug, Deserialize)]
pub struct Headers {
    #[serde(default)]
    pub host: String,
    #[serde(default, rename = "x-request-id")]
    pub x_request_id: String,
    #[serde(default, rename = "x-trace-id")]
    pub x_trace_id: String,
    #[serde(default, rename = "x-span-id")]
    pub x_span_id: String,
}

impl TheApi {
    async fn touch(&self) -> ApiResult<Payload<Headers>> {
        let req = self.get("/path/json").await?;
        send!(req, CodeDataMessage).await
    }

    async fn touch_with(
        &self,
        request_id: Option<impl ToString>,
        trace_id: Option<impl ToString>,
        span_id: Option<impl ToString>,
    ) -> ApiResult<Payload<Headers>> {
        let mut req = self.get("/path/json").await?;
        if let Some(request_id) = request_id {
            req = req.with_extension(RequestId::new(request_id.to_string()));
        }
        if let Some(trace_id) = trace_id {
            req = req.with_extension(TraceId::new(
                trace_id.to_string(),
                span_id.map(|v| v.to_string()),
            ));
        }
        send!(req, CodeDataMessage).await
    }
}

#[tokio::test]
async fn test_trace_default() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.touch().await?;
    log::debug!("res = {:?}", res);
    assert!(!res.headers.x_request_id.is_empty());
    assert!(!res.headers.x_trace_id.is_empty());
    assert_eq!(res.headers.x_request_id, res.headers.x_trace_id);

    Ok(())
}

#[tokio::test]
async fn test_trace_req() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api
        .touch_with(Some("req"), None::<&str>, None::<&str>)
        .await?;
    log::debug!("res = {:?}", res);
    assert_eq!(res.headers.x_request_id, "req");
    assert_eq!(res.headers.x_trace_id, "req");
    assert_eq!(res.headers.x_span_id, "");

    Ok(())
}

#[tokio::test]
async fn test_trace_trace() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api
        .touch_with(None::<&str>, Some("trace"), None::<&str>)
        .await?;
    log::debug!("res = {:?}", res);
    assert_eq!(res.headers.x_request_id, "trace");
    assert_eq!(res.headers.x_trace_id, "trace");
    assert_eq!(res.headers.x_span_id, "");

    Ok(())
}

#[tokio::test]
async fn test_trace_all() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.touch_with(Some("req"), Some("tr"), Some("sp")).await?;
    log::debug!("res = {:?}", res);
    assert_eq!(res.headers.x_request_id, "req");
    assert_eq!(res.headers.x_trace_id, "tr");
    assert_eq!(res.headers.x_span_id, "sp");

    Ok(())
}

#[tokio::test]
async fn test_trace_all_with_log() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().with_log(true).build();

    let res = api.touch_with(Some("req"), Some("tr"), Some("sp")).await?;
    log::debug!("res = {:?}", res);
    assert_eq!(res.headers.x_request_id, "req");
    assert_eq!(res.headers.x_trace_id, "tr");
    assert_eq!(res.headers.x_span_id, "sp");

    Ok(())
}
