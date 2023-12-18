use std::{collections::HashMap, time::Duration};

use apisdk::header::HeaderMap;
use futures::StreamExt;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::OnceCell;
use warp::{
    filters::{multipart::FormData, path::FullPath},
    reply::Reply,
    Filter,
};

pub const PORT: u16 = 3030;

#[derive(Debug, Deserialize)]
pub struct Payload<H = HashMap<String, String>> {
    pub path: String,
    pub headers: H,
    #[serde(default)]
    pub query: HashMap<String, String>,
    #[serde(default)]
    pub form: HashMap<String, String>,
}

static ONCE: OnceCell<()> = OnceCell::const_new();

pub async fn start_server() {
    ONCE.get_or_init(do_start_server).await;
}

async fn do_start_server() {
    tokio::spawn(async move {
        let dump_normal = warp::any()
            .and(warp::path!("v1" / "path" / "json"))
            .and(warp::path::full())
            .and(warp::header::headers_cloned())
            .and(warp::query())
            .and_then(handle_normal);
        let dump_form = warp::post()
            .and(warp::path!("v1" / "path" / "form"))
            .and(warp::path::full())
            .and(warp::header::headers_cloned())
            .and(warp::query())
            .and(warp::body::form())
            .and_then(handle_form);
        let dump_multipart = warp::post()
            .and(warp::path!("v1" / "path" / "multipart"))
            .and(warp::path::full())
            .and(warp::header::headers_cloned())
            .and(warp::query())
            .and(warp::multipart::form())
            .and_then(handle_multipart);

        warp::serve(dump_normal.or(dump_form).or(dump_multipart))
            .run(([127, 0, 0, 1], PORT))
            .await;
    });

    // Ensure the server is ready to work
    tokio::time::sleep(Duration::from_millis(100)).await;
}

async fn handle_normal(
    path: FullPath,
    headers: HeaderMap,
    query: HashMap<String, String>,
) -> Result<impl Reply, warp::Rejection> {
    let mut headers_map = HashMap::new();
    for (name, value) in headers {
        if let Some(name) = name {
            if let Ok(value) = value.to_str() {
                headers_map.insert(name.as_str().to_string(), value.to_string());
            }
        }
    }
    let resp = json!({
        "code": 0,
        "message": "OK",
        "data": {
            "path": path.as_str(),
            "headers": headers_map,
            "query": query,
        },
        "extra-field": "extra"
    });
    Ok(warp::reply::json(&resp))
}

async fn handle_form(
    path: FullPath,
    headers: HeaderMap,
    query: HashMap<String, String>,
    form: HashMap<String, String>,
) -> Result<impl Reply, warp::Rejection> {
    let mut headers_map = HashMap::new();
    for (name, value) in headers {
        if let Some(name) = name {
            if let Ok(value) = value.to_str() {
                headers_map.insert(name.as_str().to_string(), value.to_string());
            }
        }
    }
    let resp = json!({
        "code": 0,
        "message": "OK",
        "data": {
            "path": path.as_str(),
            "headers": headers_map,
            "query": query,
            "form": form,
        },
        "extra-field": "extra"
    });
    Ok(warp::reply::json(&resp))
}

async fn handle_multipart(
    path: FullPath,
    headers: HeaderMap,
    query: HashMap<String, String>,
    mut multipart: FormData,
) -> Result<impl Reply, warp::Rejection> {
    let mut headers_map = HashMap::new();
    for (name, value) in headers {
        if let Some(name) = name {
            if let Ok(value) = value.to_str() {
                headers_map.insert(name.as_str().to_string(), value.to_string());
            }
        }
    }
    let mut parts = HashMap::new();
    while let Some(Ok(part)) = multipart.next().await {
        parts.insert(
            part.name().to_string(),
            part.content_type()
                .map(|v| v.to_string())
                .unwrap_or_default(),
        );
    }
    let resp = json!({
        "code": 0,
        "message": "OK",
        "data": {
            "path": path.as_str(),
            "headers": headers_map,
            "query": query,
            "multipart": parts,
        },
        "extra-field": "extra"
    });
    Ok(warp::reply::json(&resp))
}
