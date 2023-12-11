use axum::{http::StatusCode, routing::get, Router};
use tower_http::services::ServeDir;

pub fn router() -> Router {
    Router::new()
        .route("/11/health", get(|| async { StatusCode::OK }))
        .nest_service("/11/assets", ServeDir::new("assets"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;

    #[tokio::test]
    async fn day11_health() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/11/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day11_recieved_png() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/11/assets/decoration.png").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers().get("content-length").unwrap(), "787297");
    }
}
