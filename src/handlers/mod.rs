use sqlx::{Pool, Postgres};

mod day0;
mod day1;
mod day11;
mod day12;
mod day13;
mod day14;
mod day15;
mod day18;
mod day19;
mod day20;
mod day21;
mod day4;
mod day5;
mod day6;
mod day7;
mod day8;

pub fn router(pool: Pool<Postgres>) -> axum::Router {
    axum::Router::new()
        .nest("/", day0::router())
        .nest("/", day1::router())
        .nest("/", day4::router())
        .nest("/", day5::router())
        .nest("/", day6::router())
        .nest("/", day7::router())
        .nest("/", day8::router())
        .nest("/", day11::router())
        .nest("/", day12::router())
        .nest("/", day13::router(pool.clone()))
        .nest("/", day14::router())
        .nest("/", day15::router())
        .nest("/", day18::router(pool))
        .nest("/", day19::router())
        .nest("/", day20::router())
        .nest("/", day21::router())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;
    use sqlx::PgPool;

    const DATABASE_URL: &str = "postgres://postgres:password@localhost:5432/shuttle";
    #[tokio::test]
    async fn day0_health() {
        let pool = PgPool::connect(DATABASE_URL).await.unwrap();
        let app = router(pool);

        let client = TestClient::new(app);
        let res = client.get("/-1/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day1_health() {
        let pool = PgPool::connect(DATABASE_URL).await.unwrap();
        let app = router(pool);

        let client = TestClient::new(app);
        let res = client.get("/1/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day4_health() {
        let pool = PgPool::connect(DATABASE_URL).await.unwrap();
        let app = router(pool);

        let client = TestClient::new(app);
        let res = client.get("/4/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day6_health() {
        let pool = PgPool::connect(DATABASE_URL).await.unwrap();
        let app = router(pool);

        let client = TestClient::new(app);
        let res = client.get("/6/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day7_health() {
        let pool = PgPool::connect(DATABASE_URL).await.unwrap();
        let app = router(pool);

        let client = TestClient::new(app);
        let res = client.get("/7/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day8_health() {
        let pool = PgPool::connect(DATABASE_URL).await.unwrap();
        let app = router(pool);

        let client = TestClient::new(app);
        let res = client.get("/8/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }
}
