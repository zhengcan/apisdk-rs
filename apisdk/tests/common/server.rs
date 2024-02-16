use std::{collections::HashMap, time::Duration};

use apisdk::{header::HeaderMap, ApiError, ResponseBody};
use futures::StreamExt;
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::json;
use tokio::sync::OnceCell;
use warp::{
    filters::{multipart::FormData, path::FullPath},
    reply::Reply,
    Filter,
};

pub const PORT: u16 = 3030;

static ONCE: OnceCell<()> = OnceCell::const_new();

pub async fn start_server() {
    ONCE.get_or_init(do_start_server).await;
}

async fn do_start_server() {
    tokio::spawn(async move {
        let dump_json = warp::path!("v1" / "path" / "json")
            .and(warp::path::full())
            .and(warp::header::headers_cloned())
            .and(warp::query())
            .and_then(handle_json);
        let dump_xml = warp::path!("v1" / "path" / "xml")
            .and(warp::path::full())
            .and(warp::header::headers_cloned())
            .and(warp::query())
            .and_then(handle_xml);
        let dump_text = warp::path!("v1" / "path" / "text")
            .and(warp::path::full())
            .and(warp::header::headers_cloned())
            .and(warp::query())
            .and_then(handle_text);
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
        let not_found = warp::path!("v1" / "not-found").and_then(handle_not_found);

        warp::serve(
            dump_json
                .or(dump_xml)
                .or(dump_text)
                .or(dump_form)
                .or(dump_multipart)
                .or(not_found),
        )
        .run(([127, 0, 0, 1], PORT))
        .await;
    });

    // Ensure the server is ready to work
    tokio::time::sleep(Duration::from_millis(200)).await;
}

async fn handle_json(
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

async fn handle_xml(
    path: FullPath,
    headers: HeaderMap,
    query: HashMap<String, String>,
) -> Result<impl Reply, warp::Rejection> {
    warp::http::Response::builder()
        .header("Content-Type", "text/xml")
        .body(
            r#"
        <xml>
            <code>0</code>
            <data>
                <hello>world</hello>
            </data>
        </xml>
        "#
            .trim(),
        )
        .map_err(|_| warp::reject())
}

async fn handle_text(
    path: FullPath,
    headers: HeaderMap,
    query: HashMap<String, String>,
) -> Result<impl Reply, warp::Rejection> {
    warp::http::Response::builder()
        .header("Content-Type", "text/plain")
        .body("text goes here")
        .map_err(|_| warp::reject())
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

async fn handle_not_found() -> Result<String, warp::Rejection> {
    Err(warp::reject::not_found())
}

#[tokio::test]
#[ignore]
async fn standalone_server() {
    start_server().await;
    tokio::time::sleep(Duration::from_secs(60 * 5)).await
}
