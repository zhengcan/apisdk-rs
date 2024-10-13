//! A highlevel API client framework for Rust.

mod core;
pub mod digest;
mod executor;
mod extension;
mod extractor;
mod result;
mod url;

pub use crate::core::*;
pub use crate::executor::*;
pub use crate::extension::*;
pub use crate::extractor::*;
pub use crate::result::*;
pub use crate::url::*;

// Re-export macros
pub use apisdk_macros::*;

// Re-export from async_trait::async_trait
pub use async_trait::async_trait;

/// Re-export serde_json
pub use serde_json;

/// Re-export quick_xml
pub use quick_xml;

// Re-export reqwest types
pub use reqwest::dns;
pub use reqwest::header;
pub use reqwest::multipart;
pub use reqwest::ClientBuilder;
pub use reqwest::IntoUrl;
pub use reqwest::Method;
pub use reqwest::Request;
pub use reqwest::Response;
pub use reqwest::Url;

// Re-export reqwest_middleware types
pub use reqwest_middleware::ClientWithMiddleware as Client;
pub use reqwest_middleware::Error as MiddlewareError;
pub use reqwest_middleware::Extension;
pub use reqwest_middleware::Middleware;
pub use reqwest_middleware::Next;
pub use reqwest_middleware::RequestBuilder;
pub use reqwest_middleware::RequestInitialiser as Initialiser;

// Re-export http types
pub use http::Extensions;

/// Re-export log::LevelFilter
pub use log::LevelFilter;
