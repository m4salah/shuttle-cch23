use axum::{http::StatusCode, response::IntoResponse, routing::get};

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/", get(hello_world))
        .route("/-1/error", get(internal_server_error))
        .route("/-1/health", get(|| async { StatusCode::OK }))
}

async fn internal_server_error() -> impl IntoResponse {
    StatusCode::INTERNAL_SERVER_ERROR
}

async fn hello_world() -> impl IntoResponse {
    "Hello, World!"
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;

    #[tokio::test]
    async fn hello_world() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await, "Hello, World!");
    }

    #[tokio::test]
    async fn internal_server_error() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/-1/error").send().await;
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(res.text().await, "");
    }
}
