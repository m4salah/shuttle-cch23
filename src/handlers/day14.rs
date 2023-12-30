use askama::Template;
use axum::{
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

pub fn router() -> Router {
    Router::new()
        .route("/14/health", get(|| async { StatusCode::OK }))
        .route("/14/unsafe", post(unsafe_santa))
        .route("/14/safe", post(safe_santa))
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Template)]
#[template(path = "day14.html", escape = "none")]
struct UnsafeHtmlContent {
    pub content: String,
}

async fn unsafe_santa(
    Json(content): Json<UnsafeHtmlContent>,
) -> Result<(StatusCode, Html<String>), StatusCode> {
    println!("{content:?}");
    let reply_html = UnsafeHtmlContent {
        content: content.content,
    }
    .render()
    .map_err(|e| {
        tracing::error!("error while rendering html {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok((StatusCode::OK, Html(reply_html)))
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Template)]
#[template(path = "day14.html")]
struct SafeHtmlContent {
    pub content: String,
}

async fn safe_santa(
    Json(content): Json<UnsafeHtmlContent>,
) -> Result<(StatusCode, Html<String>), StatusCode> {
    println!("{content:?}");
    let reply_html = SafeHtmlContent {
        content: content.content,
    }
    .render()
    .map_err(|e| {
        tracing::error!("error while rendering html {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok((StatusCode::OK, Html(reply_html)))
}
