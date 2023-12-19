use apisdk::{AccessTokenSignature, ApiResult, ApiRouters};

use crate::common::{init_logger, TheApi};

mod common;

#[tokio::test]
async fn test_derive_with_router() -> ApiResult<()> {
    init_logger();

    let api = TheApi::builder()
        .with_router(ApiRouters::fixed(("127.0.0.1", 3030)))
        .with_signature(AccessTokenSignature::new("access_token"))
        .build();
    println!("api = {:?}", api);

    Ok(())
}
