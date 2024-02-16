use apisdk::ApiResult;
use apisdk_macros::http_api;
use common::init_logger;

mod common;

#[http_api("http://localhost:3030/v1", no_default)]
#[derive(Debug)]
struct ComplexApi {
    something_must_init: String,
}

impl ComplexApi {
    fn new(sth: impl ToString) -> Self {
        Self {
            core: Self::builder().build_core(),
            something_must_init: sth.to_string(),
        }
    }
}

#[tokio::test]
async fn test_ctor() -> ApiResult<()> {
    init_logger();

    let api = ComplexApi::new("");
    log::debug!("api = {:?}", api);

    Ok(())
}
