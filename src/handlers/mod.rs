mod day0;
mod day1;
mod day11;
mod day12;
mod day4;
mod day6;
mod day7;
mod day8;

pub fn router() -> axum::Router {
    axum::Router::new()
        .nest("/", day0::router())
        .nest("/", day1::router())
        .nest("/", day4::router())
        .nest("/", day6::router())
        .nest("/", day7::router())
        .nest("/", day8::router())
        .nest("/", day11::router())
        .nest("/", day12::router())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;

    #[tokio::test]
    async fn day0_health() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/-1/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day1_health() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/1/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day4_health() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/4/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day6_health() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/6/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day7_health() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/7/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day8_health() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/8/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }
}
