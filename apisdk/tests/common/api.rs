use apisdk::http_api;

/// This a sample API
#[http_api("http://localhost:3030/v1")]
#[derive(Debug, Clone)]
pub struct TheApi;
