# API SDK

[English](README.md) ∙ [简体中文](README.zh-CN.md)

An easy-to-use API toolkit for writing HTTP API Clients for Rust.

- Built on top of [reqwest](https://github.com/seanmonstar/reqwest/) to handle HTTP requests
- Macros to define API and send requests
- Send request as JSON / XML / form / multipart
- Parse response by [serde](https://serde.rs/)
    - Use [serde_json](https://github.com/serde-rs/json) to process JSON response
    - Use [quick-xml](https://github.com/tafia/quick-xml) to process XML response
- Support `X-Request-ID` and `X-Trace-ID`/`X-Span-ID`
- More customization capabilities
    - Provide `UrlRewriter` and `DnsResolver` to customize URL and API endpoint
    - Set `Authorization` header by using `ApiSignature`
    - Provide middlewares by integrate [reqwest-middleware](https://github.com/TrueLayer/reqwest-middleware/)
    - Mock server response by using `MockServer`
- [Changelog](CHANGELOG.md)

# Motivation

When using [reqwest](https://github.com/seanmonstar/reqwest/) to send API requests to server side, we have to do some common work. Including setting authentication information, parsing request responses, handle exceptions, adding log and tracking information, etc.

For this reason, we often develop some auxiliary functions to achieve the above functions. The design purpose of this crate is to simplify this part of the development work and provide a common design implementation.

# Getting Started

### Install

Update `Cargo.toml` to add this crate as dependency.

```toml
[dependencies]
apisdk = { version = "0.0.6" }
```

This crate has several features:
- uuid
    - use [`uuid`](https://crates.io/crates/uuid) instead of [`nanoid`](https://crates.io/crates/nanoid) to generate `X-Request-ID` and `X-Trace-ID`
- dns
    - install [`hickory-resolver`](https://crates.io/crates/hickory-resolver) (aka. [`trust-dns-resolver`](https://crates.io/crates/trust-dns-resolver)), and able to use it to do DNS queries

### Define API struct

To define a very simple API, we just need a few lines of code.

```rust
use apisdk::{http_api, send, ApiResult};

// Define an API struct
#[http_api("https://www.example.com/api")]
#[derive(Debug, Clone)] // optional
pub struct MyApi;

// Response DTO
#[derive(serde::Deserialize)]
pub struct User {}

impl MyApi {
    // Define a function for public use.
    // It should return ApiResult<T>, which is an alias for Result<T, ApiError>.
    pub async fn get_user(&self, user_id: u64) -> ApiResult<User> {
        // Initiate a GET request with the URL path, and wait for the endpoint to be resolved.
        let req = self.get(format!("/user/{}", user_id)).await?;

        // Send the request to server, and parse it to result.
        send!(req).await
    }
}
```

### Call APIs

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

### create API instance

We can use `XxxApi::builder()` to get an instance of `ApiBuilder`, and call following functions to customize API instance. 

- `with_client`
    - set `reqwest::ClientBuilder` to customize Client
- `with_rewriter`
    - rewrite HTTP Url
- `with_resolver`
    - custom DNS queries
- `with_signature`
    - set credentials for each request
- `with_initialiser` & `with_middleware`
    - support all `reqwest-middleware` components
- `with_log`
    - enable/disable logs in processing requests

After that, we should call `build()` to create the API instance.

For really simple APIs, we can use `XxxApi::default()` to replace `XxxApi::builder().build()`.

### create HTTP request

The API instances provide a series of functions to assist in creating HTTP requests.

- create with HTTP method
    - `async fn request(method: Method, path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
- convenience functions
    - `async fn head(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn get(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn post(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn put(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn patch(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn delete(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn options(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn trace(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`

We can also use the `core` field of the API instance to access more low-level functionality.

```rust
let api = XxxApi::default();
let req = api.core  // an instance of apisdk::ApiCore
    .rebase("http://different.host.com/api")  // reset the BaseUrl
    .build_request(Method::GET, "/path")?;
```

### extends `RequestBuilder`

This crate re-export `RequestBuilder` from `reqwest-middleware`, and provides several useful extensions. We may use `req.with_extension()` to apply these extensions.

- `RequestId`
    - set value of `X-Request-ID`
- `TraceId`
    - set value of `X-Trace-ID` and/or `X-Span-ID`
- `MockServer`
    - mock the server response

### `send` macros

- `send`
    - send request, and not detect or process the payload
- `send_json`
    - send request with JSON payload
- `send_xml`
    - send request with XML payload
- `send_form`
    - send request with urlencoded form or multipart form
- `send_multipart`
    - send request with multipart form

These macros support following forms.

```rust
// Form 1: send and parse response as JSON / XML
let _: Data = send!(req).await?;

// Form 2: send, drop response and return ApiResult<()>
send!(req, ()).await?;

// Form 3: send and return ResponseBody
let _ = send!(req, Body).await?;

// Form 4: send and parse JSON response to Data
let _: Data = send!(req, Json).await?;

// Form 5: send and parse XML response to Data
let _: Data = send!(req, Xml).await?;

// Form 6: send and parse Text response to Data by using FromStr trait
let _: Data = send!(req, Text).await?;

// Form 7: send and parse JSON response to Data
let _ = send!(req, Data).await?;

// Form 8: send and parse JSON response to Data
let _ = send!(req, Json<Data>).await?;
```

You may check `tests` for more examples.
