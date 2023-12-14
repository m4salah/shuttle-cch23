use std::env;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;

#[derive(Clone)]
struct SqlState {
    pool: PgPool,
}

pub async fn router() -> Router {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set the env variable");
    let pool = PgPool::connect(&database_url).await.unwrap();

    Router::new()
        .route("/13/health", get(|| async { StatusCode::OK }))
        .route("/13/sql", get(sequal))
        .route("/13/reset", post(reset))
        .route("/13/orders", post(create_orders))
        .route("/13/orders/total", get(total_orders))
        .route("/13/orders/popular", get(popular))
        .with_state(SqlState { pool })
}

async fn sequal(State(state): State<SqlState>) -> Result<String, StatusCode> {
    tracing::info!("sql orders called");
    let row = sqlx::query!("SELECT 20231213 number")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("error while fetching from database {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .number
        .ok_or_else(|| {
            tracing::error!("error while fetching from making it number");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    tracing::info!("row in sql {row}");
    Ok(format!("{row}"))
}

async fn reset(State(state): State<SqlState>) -> Result<(), StatusCode> {
    // drop orders table if exists
    tracing::info!("reset orders called");
    sqlx::query!("DROP TABLE IF EXISTS orders;")
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("error droping orders table {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // create order table
    sqlx::query!(
        r#"
        CREATE TABLE orders (
            id INT PRIMARY KEY,
            region_id INT,
            gift_name VARCHAR(50),
            quantity INT
        );
    "#
    )
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("error creating orders table {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(())
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Order {
    pub id: i32,
    pub region_id: i32,
    pub gift_name: String,
    pub quantity: i32,
}

async fn create_orders(
    State(state): State<SqlState>,
    Json(orders): Json<Vec<Order>>,
) -> Result<(), StatusCode> {
    tracing::info!("create orders called {orders:?}");
    // create order table
    let transaction = state.pool.begin().await.map_err(|e| {
        tracing::error!("error starting transaction {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    for order in orders {
        sqlx::query!(
            "insert into orders(id, region_id, gift_name, quantity) values($1, $2, $3, $4)",
            order.id,
            order.region_id,
            order.gift_name,
            order.quantity,
        )
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("error inserting to the database {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }
    transaction.commit().await.map_err(|e| {
        tracing::error!("error commiting the transaction {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(())
}

async fn total_orders(State(state): State<SqlState>) -> Result<Json<Value>, StatusCode> {
    tracing::info!("total orders called");
    let total = sqlx::query!("select sum(quantity) from orders")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("error creating orders table {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .sum
        .unwrap_or_default();
    Ok(Json(json!({ "total": total})))
}

async fn popular(State(state): State<SqlState>) -> Result<Json<Value>, StatusCode> {
    tracing::info!("popular called");
    let popular =
        sqlx::query!("select sum(quantity) as quantity, gift_name from orders group by gift_name")
            .fetch_all(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!("error creating orders table {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .into_iter()
            .max_by_key(|p| p.quantity);
    if let Some(p) = popular {
        Ok(Json(json!({ "popular": p.gift_name})))
    } else {
        Ok(Json(json!({ "popular": Value::Null})))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;
    use reqwest::header::CONTENT_TYPE;
    use serde_json::{json, Value};

    #[tokio::test]
    async fn day13_health() {
        let app = router().await;

        let client = TestClient::new(app);
        let res = client.get("/13/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day13_sql() {
        let app = router().await;

        let client = TestClient::new(app);
        let res = client.get("/13/sql").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await, "20231213");
    }

    #[tokio::test]
    async fn day13_reset() {
        let app = router().await;

        let client = TestClient::new(app);
        let res = client.get("/13/reset").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day13_create_orders() {
        let app = router().await;

        let client = TestClient::new(app);
        let res = client.get("/13/reset").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        let res = client
            .post("/13/orders")
            .body(
                json!(
                [
                    {"id":1,"region_id":2,"gift_name":"Toy Train","quantity":5},
                    {"id":2,"region_id":2,"gift_name":"Doll","quantity":8},
                    {"id":3,"region_id":3,"gift_name":"Action Figure","quantity":12},
                    {"id":4,"region_id":4,"gift_name":"Board Game","quantity":10},
                    {"id":5,"region_id":2,"gift_name":"Teddy Bear","quantity":6},
                    {"id":6,"region_id":3,"gift_name":"Toy Train","quantity":3}
                  ]
                        )
                .to_string(),
            )
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day13_total() {
        let app = router().await;

        let client = TestClient::new(app);
        let res = client.get("/13/reset").send().await;
        assert_eq!(res.status(), StatusCode::OK);

        let res = client
            .post("/13/orders")
            .body(
                json!(
                [
                    {"id":1,"region_id":2,"gift_name":"Toy Train","quantity":5},
                    {"id":2,"region_id":2,"gift_name":"Doll","quantity":8},
                    {"id":3,"region_id":3,"gift_name":"Action Figure","quantity":12},
                    {"id":4,"region_id":4,"gift_name":"Board Game","quantity":10},
                    {"id":5,"region_id":2,"gift_name":"Teddy Bear","quantity":6},
                    {"id":6,"region_id":3,"gift_name":"Toy Train","quantity":3}
                  ]
                        )
                .to_string(),
            )
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);

        let res = client.get("/13/orders/total").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.json::<Value>().await, json!({"total":44}));
    }
}
