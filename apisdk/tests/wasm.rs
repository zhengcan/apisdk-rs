#![cfg(target_arch = "wasm32")]

use apisdk::{send, ApiResult};
use apisdk_macros::http_api;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[http_api("https://example.com/")]
struct ExampleApi;

impl ExampleApi {
    async fn home(&self) -> ApiResult<String> {
        let req = self.get("/").await?;
        send!(req).await
    }
}

#[wasm_bindgen_test]
async fn test_example_home() {
    let api = ExampleApi::default();
    let res = api.home().await.expect("http get example");
    log(&format!("Result: {}", res));
}
