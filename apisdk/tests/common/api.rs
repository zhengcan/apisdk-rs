use std::collections::HashMap;

use apisdk::{http_api, ApiError, ResponseBody};
use serde::{de::DeserializeOwned, Deserialize};

/// This a sample API
#[http_api("http://localhost:3030/v1")]
#[derive(Debug, Clone)]
pub struct TheApi;

#[derive(Debug, Deserialize)]
pub struct Payload<H = HashMap<String, String>> {
    pub path: String,
    pub headers: H,
    #[serde(default)]
    pub query: HashMap<String, String>,
    #[serde(default)]
    pub form: HashMap<String, String>,
}

impl<H> TryFrom<ResponseBody> for Payload<H>
where
    H: DeserializeOwned,
{
    type Error = ApiError;

    fn try_from(body: ResponseBody) -> Result<Self, Self::Error> {
        body.parse_json()
    }
}
