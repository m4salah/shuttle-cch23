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

#[cfg(test)]
mod tests {

    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;
    use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};
    use serde_json::json;

    #[tokio::test]
    async fn day14_health() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/14/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day14_unsafe_santa() {
        let app = router();

        let client = TestClient::new(app);
        let res = client
            .post("/14/unsafe")
            .body(
                json!({
                "content": "<h1>Welcome to the North Pole!</h1>"
                })
                .to_string(),
            )
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await;
        assert_eq!(
            res.headers().get(CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
        assert_eq!(res.headers().get(CONTENT_LENGTH).unwrap(), "124");
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day14_safe_santa() {
        let app = router();

        let client = TestClient::new(app);
        let res = client
            .post("/14/safe")
            .body(
                json!({
                "content": "<script>alert(\"XSS Attack!\")</script>"
                })
                .to_string(),
            )
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_eq!(
            res.headers().get(CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
        assert_eq!(res.status(), StatusCode::OK);
    }
}
