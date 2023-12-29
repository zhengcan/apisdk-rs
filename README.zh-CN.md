# API SDK

[English](README.md) ∙ [简体中文](README.zh-CN.md)

一个用于为 Rust 编写 HTTP API 客户端的，易于使用的 API 工具包。

- 使用 [reqwest](https://github.com/seanmonstar/reqwest/) 来处理 HTTP 请求
- 提供一系列用于定义 API 和发送请求的宏
- 支持以 JSON / XML / 表单 / Multipart 等形式发送请求体
- 支持 [serde](https://serde.rs/) 解析响应
    - 使用 [serde_json](https://github.com/serde-rs/json) 处理 JSON 响应
    - 使用 [quick-xml](https://github.com/tafia/quick-xml) 处理 XML 响应
- 支持 `X-Request-ID` 和 `X-Trace-ID`/`X-Span-ID`
- 更多自定义能力
    - 提供 `UrlRewriter` 和 `DnsResolver` 用于定制 URL 和 API 端点
    - 可以使用 `ApiSignature` 设置 `Authorization` 鉴权头
    - 通过集成 [reqwest-middleware](https://github.com/TrueLayer/reqwest-middleware/) 来提供中间件能力
    - 可以使用 `MockServer` 来提供仿冒服务器端响应
- [变更日志](CHANGELOG.md)

# 开发动机

在使用 [reqwest](https://github.com/seanmonstar/reqwest/) 向服务器端发送 API 请求时，我们经常需要做一个通用的工作。包括设置鉴权信息、解析服务器端响应、处理异常、添加日志和追踪信息等。

为此，我们经常会开发一些辅助功能来实现上述功能。 这个 crate 的设计目的是简化这部分的开发工作，提供通用的设计实现。

# 入门

### 安装

更新 `Cargo.toml` 来将本 crate 添加为依赖。

```toml
[dependencies]
apisdk = { version = "0.0.6" }
```

本 crate 包含以下特性：
- uuid
    - 使用 [`uuid`](https://crates.io/crates/uuid) 替代 [`nanoid`](https://crates.io/crates/nanoid) 来生成 `X-Request-ID` 和 `X-Trace-ID`
- dns
    - 安装 [`hickory-resolver`](https://crates.io/crates/hickory-resolver) (别名 [`trust-dns-resolver`](https://crates.io/crates/trust-dns-resolver))，且支持将其用于 DNS 查询

### 定义 API 对象

如果需要定义一个非常简单的 API，我们仅需要若干行代码。

```rust
use apisdk::{http_api, send, ApiResult};

// 定义一个 API 结构体
#[http_api("https://www.example.com/api")]
#[derive(Debug, Clone)] // 可选
pub struct MyApi;

// 响应 DTO
#[derive(serde::Deserialize)]
pub struct User {}

impl MyApi {
    // 定义一个可公开使用的方法
    // 它应该返回 ApiResult<T>，其是 Result<T, ApiError> 的别名
    pub async fn get_user(&self, user_id: u64) -> ApiResult<User> {
        // 使用一个 URL 路径来初始化一个 GET 请求，并等待服务端点解析完成
        let req = self.get(format!("/user/{}", user_id)).await?;

        // 发送请求到服务器，并将其解析为函数返回
        send!(req).await
    }
}
```

为了使用该 API，需要使用以下几步。

```rust
use apisdk::ApiResult;

async fn foo() -> ApiResult<()> {
    // 使用默认设置来初始化 API 实例
    // 或者使用 MyApi::builder().build() 来生成一个定制化实例
    let api = MyApi::default();

    // 调用其上的方法来执行 HTTP 请求
    let user = api.get_user(1).await?;
    
    Ok(())
}
```

# 关键要点

### `http_api` 与 `api_method` 宏

- `http_api`
    - 用于将一个结构体声明为 API
    - `#[http_api("https://api.site/base")]`
- `api_method`
    - (可选) 精化一个 API 方法

### 创建 API 实例

我们可以使用 `XxxApi::builder()` 来获得一个 `ApiBuilder` 实例，再调用一下方法对 API 实例进行定制化。

- `with_client`
    - 传入 `reqwest::ClientBuilder` 来定制化底层 Client
- `with_rewriter`
    - 重写 HTTP Url
- `with_resolver`
    - 自定义 DNS 查询
- `with_signature`
    - 为每个请求设置身份信息
- `with_initialiser` & `with_middleware`
    - 支持所有 `reqwest-middleware` 组件
- `with_log`
    - 启用/禁用请求处理过程中的日志

定制完成之后，再调用 `build()` 来创建 API 实例。

对于确实很简单的 API，也可以使用 `XxxApi::default()` 来提代 `XxxApi::builder().build()`。

### 创建 HTTP 请求

API 实例提供了一系列方法用于辅助创建 HTTP 请求。

- 指定 HTTP Method 来创建
    - `async fn request(method: Method, path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
- 便捷方法
    - `async fn head(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn get(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn post(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn put(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn patch(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn delete(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn options(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`
    - `async fn trace(path: impl AsRef<str>) -> ApiResult<RequestBuilder>`

我们可以访问 API 实例的 `core` 字段来获取更多底层功能，如：

```rust
let api = XxxApi::default();
let req = api.core  // 一个 apisdk::ApiCore 实例
    .rebase("http://different.host.com/api")  // 重新设置 BaseUrl
    .build_request(Method::GET, "/path")?;
```

### 扩展 `RequestBuilder`

本 crate 重新导出了 `reqwest-middleware` 的 `RequestBuilder`，并提供了一系列有用扩展。我们可以使用 `req.with_extension()` 来应用这些扩展。

- `RequestId`
    - 设置 `X-Request-ID`
- `TraceId`
    - 设置 `X-Trace-ID` 和/或 `X-Span-ID`
- `MockServer`
    - 仿冒服务器端响应

### `send` 宏

- `send`
    - 发送请求（不检测和处理请求负载）
- `send_json`
    - 以 JSON 为请求体发送请求
- `send_xml`
    - 以 XML 为请求体发送请求
- `send_form`
    - 发送 urlencoded 或者 multipart 表单
- `send_multipart`
    - 发送 multipart 表单

这些宏均支持以下形式。

```rust
// 形式 1: 发送请求，并将响应以 JSON 或 XML 格式进行解析为 Data 类型
let _: Data = send!(req).await?;

// 形式 2: 发送请求，丢弃响应在返回 ApiResult<()>
send!(req, ()).await?;

// 形式 3: 发送请求，并返回 ResponseBody
let _ = send!(req, Body).await?;

// 形式 4: 发送请求，并将响应以 JSON 格式进行解析为 Data 类型
let _: Data = send!(req, Json).await?;

// 形式 5: 发送请求，并将响应以 XML 格式进行解析为 Data 类型
let _: Data = send!(req, Xml).await?;

// 形式 6: 发送请求，并将响应通过 FromStr 特征进行解析为 Data 类型
let _: Data = send!(req, Text).await?;

// 形式 7: 发送请求，并将响应以 JSON 格式进行解析为 Data 类型
let _ = send!(req, Data).await?;

// 形式 8: 发送请求，并将响应以 JSON 格式进行解析为 Data 类型
let _ = send!(req, Json<Data>).await?;
```

你可以查看 `tests` 来找到更多示例。
