use apisdk::{send, ApiResult, CodeDataMessage};
use serde::Deserialize;

use crate::common::{init_logger, start_server, TheApi};

mod common;

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct XmlData {
    code: i64,
    data: DataNode,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct DataNode {
    hello: String,
}

impl TheApi {
    async fn get_json_as_auto(&self) -> ApiResult<CodeDataMessage> {
        let req = self.get("/path/json").await?;
        send!(req).await
    }

    async fn get_xml_as_auto(&self) -> ApiResult<XmlData> {
        let req = self.get("/path/xml").await?;
        send!(req).await
    }
}

#[tokio::test]
async fn test_extract_json_as_auto() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.get_json_as_auto().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_extract_xml_as_auto() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.get_xml_as_auto().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}
