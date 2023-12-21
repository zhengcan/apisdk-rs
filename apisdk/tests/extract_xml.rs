use apisdk::{send, ApiResult};
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
    async fn get_xml_2_string(&self) -> ApiResult<String> {
        let req = self.get("/path/xml").await?;
        send!(req, Xml).await
    }

    async fn get_xml_2_data(&self) -> ApiResult<XmlData> {
        let req = self.get("/path/xml").await?;
        send!(req, Xml).await
    }
}

#[tokio::test]
async fn test_extract_xml_string() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.get_xml_2_string().await?;
    log::debug!("res = {:?}", res);
    assert!(res.contains("<xml>"));

    Ok(())
}

#[tokio::test]
async fn test_extract_xml_data() -> ApiResult<()> {
    init_logger();
    start_server().await;

    let api = TheApi::builder().build();

    let res = api.get_xml_2_data().await?;
    log::debug!("res = {:?}", res);

    Ok(())
}
