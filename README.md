# API SDK

A highlevel API client framework for Rust.

- Built on top of [reqwest](https://github.com/seanmonstar/reqwest/) to handle HTTP requests
- Macros to define API and send requests
- Send request as JSON / form / multipart
- Parse response by [serde](https://serde.rs/)
    - Use [serde_json](https://github.com/serde-rs/json) to process JSON response
    - Use [quick-xml](https://github.com/tafia/quick-xml) to process XML response
- Support `X-Request-ID` and `X-Trace-ID`/`X-Span-ID`
- More customizations
    - Rewrite `host` and `port` of URLs by using `ApiRouter`
    - Set `Authorization` header by using `ApiSignature`
    - Provide middlewares by integrate [reqwest-middleware](https://github.com/TrueLayer/reqwest-middleware/)
    - Mock server response by using `MockServer`
- [Changelog](CHANGELOG.md)

# Motivation

When using [reqwest](https://github.com/seanmonstar/reqwest/) to send API requests to server side, we have to do some common work. Including setting authentication information, parsing request responses, handle exceptions, adding log and tracking information, etc.

For this reason, we often develop some auxiliary functions to achieve the above functions. The design purpose of this crate is to simplify this part of the development work and provide a common design implementation.

# Get Start

To define a very simple API, we just need a few lines of code.

```rust
use apisdk::{http_api, send, ApiResult};

// Define an API struct
#[http_api("https://www.example.com/api")]
#[derive(Debug, Clone)] // optional
pub struct MyApi;

#[derive(serde::Deserialize)]
pub struct User {}

impl MyApi {
    // Declare a function for public use.
    // It should return ApiResult<T>, which is an alias for Result<T, ApiError>.
    pub async fn get_user(&self, user_id: u64) -> ApiResult<User> {
        // Initiate a GET request with the URL path, and wait for the endpoint to be resolved.
        let req = self.get(format!("/user/{}", user_id)).await?;

        // Send the request to server, and parse it to result.
        send!(req).await
    }
}
```

To use the API, just follow these steps.

```rust
use apisdk::ApiResult;

async fn foo() -> ApiResult<()> {
    // Initiate an API instance with default settings.
    // Or use MyApi::builder().build() to generate a customized instance.
    let api = MyApi::default();

    // Invoke the function to execute HTTP request.
    let user = api.get_user(1).await?;
    
    Ok(())
}
```

# Key Points

### `http_api` and `api_method` macros

- `http_api`
    - declare a struct as an API
    - `#[http_api("https://api.site/base")]`
- `api_method`
    - (optional) refine an API method

### customize API instance

We can use `XxxApi::builder()` to get an instance of `ApiBuilder`, and call following functions to customize API instance. 

- `with_router`
    - rewrite host and port
- `with_signature`
    - set credentials for each request
- `with_initialiser` & `with_middleware`
    - support all `reqwest-middleware` components
- `with_log`
    - enable/disable logs in processing requests

### create HTTP request

The `http_api` defines several functions for `XxxApi` to create HTTP request. 

- create with HTTP method
    - `async fn request(method: Method, path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
- quick functions
    - `async fn head(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn get(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn post(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn put(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn patch(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn delete(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn options(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn trace(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`

### extends `RequestBuilder`

This crate re-export `RequestBuilder` from `reqwest-middleware`, and provides several useful extensions. We may use `req.with_extension()` to apply these extensions.

- `RequestId`
    - set value of `X-Request-ID`
- `TraceId`
    - set value of `X-Trace-ID` and/or `X-Span-ID`
- `MockServer`
    - mock the server response by using `serde_json::Value`

### `send` macros

- `send`
    - send request, and not detect or process the payload
- `send_json`
    - send request with JSON payload
- `send_form`
    - send request with urlencoded form or multipart form
- `send_multipart`
    - send request with multipart form

```
// Form 1: send and parse JSON response to Data
let _: Data = send!(req).await;

// Form 2: send and parse JSON response to Data
let _ = send!(req, Data).await;

// Form 3: send and parse JSON response to Data
let _: Data = send!(req, Json).await;

// Form 4: send and parse XML response to Data
let _: Data = send!(req, Xml).await;

// Form 5: send and parse Text response to Data by using FromStr trait
let _: Data = send!(req, Text).await;

// Form 6: send and parse JSON response to Data
let _ = send!(req, Json<Data>).await;

// Form 7: send, drop response and return ApiResult<()>
send!(req, ()).await;
```

