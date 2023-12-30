use std::collections::HashSet;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

pub fn router() -> Router {
    Router::new()
        .route("/22/health", get(|| async { StatusCode::OK }))
        .route("/22/integers", post(integers))
        .route("/22/rocket", post(rocket))
}

async fn integers(content: String) -> impl IntoResponse {
    let mut result: HashSet<usize> = HashSet::new();
    for line in content.lines() {
        if let Ok(num) = line.trim().parse::<usize>() {
            if result.take(&num).is_none() {
                result.insert(num);
            }
        }
    }
    let rep = result.into_iter().next().unwrap_or_default();
    "🎁".repeat(rep)
}

async fn rocket(content: String) -> impl IntoResponse {
    let mut result: HashSet<usize> = HashSet::new();
    for line in content.lines() {
        if let Ok(num) = line.trim().parse::<usize>() {
            if result.take(&num).is_none() {
                result.insert(num);
            }
        }
    }
    let gifts_count = result.into_iter().next().unwrap_or_default();
    "🎁".repeat(gifts_count)
}
#[cfg(test)]
mod tests {

    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;

    #[tokio::test]
    async fn day22_health() {
        let app = router();
        let client = TestClient::new(app);
        let res = client.get("/22/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day22_integers() {
        let app = router();
        let client = TestClient::new(app);
        let res = client
            .post("/22/integers")
            .body(
                r#"888
77 
888
22
77"#,
            )
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await, "🎁".repeat(22));
    }

    #[tokio::test]
    async fn day22_integers_68() {
        let app = router();
        let client = TestClient::new(app);
        let res = client
            .post("/22/integers")
            .body(
                "\
11111111111111111111
555555555555555
33333333
68
555555555555555
33333333
4444
11111111111111111111
4444
",
            )
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await, "🎁".repeat(68));
    }
}
