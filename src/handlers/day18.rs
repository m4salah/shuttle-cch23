use std::env;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::day13::{create_orders, SqlState};

pub async fn router() -> Router {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set the env variable");
    let pool = PgPool::connect(&database_url).await.unwrap();

    Router::new()
        .route("/18/health", get(|| async { StatusCode::OK }))
        .route("/18/reset", post(reset))
        .route("/18/orders", post(create_orders))
        .route("/18/regions", post(create_regions))
        .route("/18/regions/total", get(total_per_region))
        .route("/18/regions/top_list/:q", get(top_list))
        .with_state(SqlState { pool })
}

async fn reset(State(state): State<SqlState>) -> Result<(), StatusCode> {
    // drop orders table if exists
    tracing::info!("reset orders called");

    let transaction = state.pool.begin().await.map_err(|e| {
        tracing::error!("error starting transaction {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // drop regions table
    sqlx::query!("DROP TABLE IF EXISTS regions;")
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("error droping regions table {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // drop orders table
    sqlx::query!("DROP TABLE IF EXISTS orders;")
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("error droping orders table {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // create regions table
    sqlx::query!(
        r#"
        CREATE TABLE regions (
            id INT PRIMARY KEY,
            name VARCHAR(50)
        )"#
    )
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("error creating regions table {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // create orders table
    sqlx::query!(
        r#"
        CREATE TABLE orders (
            id INT PRIMARY KEY,
            region_id INT,
            gift_name VARCHAR(50),
            quantity INT
        );
    "#,
    )
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("error creating orders table {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    transaction.commit().await.map_err(|e| {
        tracing::error!("error commiting the transaction {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(())
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Region {
    pub id: i32,
    pub name: String,
}

pub async fn create_regions(
    State(state): State<SqlState>,
    Json(regions): Json<Vec<Region>>,
) -> Result<(), StatusCode> {
    tracing::info!("create regions called {regions:?}");
    let transaction = state.pool.begin().await.map_err(|e| {
        tracing::error!("error starting transaction {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    for region in regions {
        sqlx::query!(
            "insert into regions(id, name) values($1, $2)",
            region.id,
            region.name
        )
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("error inserting regions to the database {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }
    transaction.commit().await.map_err(|e| {
        tracing::error!("error commiting the transaction {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(())
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct TotalPerRegion {
    pub region: String,
    pub total: i64,
}
pub async fn total_per_region(
    State(state): State<SqlState>,
) -> Result<Json<Vec<TotalPerRegion>>, StatusCode> {
    tracing::info!("total per regions called");
    let orders = sqlx::query_as!(
        TotalPerRegion,
        r#"
        select 
            name as "region!", 
            sum(quantity)::INT as "total!" 
        from 
            orders o 
        inner join regions r on o.region_id = r.id 
        group by 
            region_id, name
        order by
            name
            ;
    "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("error getting total per region from database {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(orders))
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct TopListResult {
    pub region: String,
    pub top_gifts: Vec<String>,
}
pub async fn top_list(
    State(state): State<SqlState>,
    Path(top): Path<i64>,
) -> Result<Json<Vec<TopListResult>>, StatusCode> {
    tracing::info!("top list called with {top}");
    let regions = sqlx::query_as!(TopListResult,
        r#"select r.name as "region!", COALESCE(NULLIF(ARRAY_AGG(o.gift_name), '{NULL}'), '{}'::text[]) AS "top_gifts!" from regions r left join LATERAL  
        (
            select gift_name, sum(quantity) as sum_quantity from orders where r.id = region_id group by gift_name, region_id order by sum_quantity desc limit $1
        ) o on true group by r.name order by r.name
         ;"#, top
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("error getting total per region from database {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    println!("{regions:?}");
    Ok(Json(regions))
}

#[cfg(test)]
mod tests {

    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;
    use reqwest::header::CONTENT_TYPE;
    use serde_json::{json, Value};

    #[tokio::test]
    async fn day18_health() {
        let app = router().await;

        let client = TestClient::new(app);
        let res = client.get("/18/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day18_reset() {
        let app = router().await;

        let client = TestClient::new(app);
        let res = client.post("/18/reset").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day18_create_orders() {
        let app = router().await;

        let client = TestClient::new(app);
        let res = client.post("/18/reset").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        let res = client
            .post("/18/orders")
            .body(
                json!(
                [
                    {"id":1,"region_id":2,"gift_name":"Board Game","quantity":5},
                    {"id":2,"region_id":2,"gift_name":"Origami Set","quantity":8},
                    {"id":3,"region_id":3,"gift_name":"Action Figure","quantity":12},
                    {"id":4,"region_id":4,"gift_name":"Teddy Bear","quantity":10},
                    {"id":5,"region_id":2,"gift_name":"Yarn Ball","quantity":6},
                    {"id":6,"region_id":3,"gift_name":"Art Set","quantity":3},
                    {"id":7,"region_id":5,"gift_name":"Robot Lego Kit","quantity":5},
                    {"id":8,"region_id":6,"gift_name":"Drone","quantity":9}
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
    async fn day18_create_regions() {
        let app = router().await;

        let client = TestClient::new(app);
        let res = client.post("/18/reset").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        let res = client
            .post("/18/regions")
            .body(
                json!(
                [
                    {"id":1,"name":"North Pole"},
                    {"id":2,"name":"Europe"},
                    {"id":3,"name":"North America"},
                    {"id":4,"name":"South America"},
                    {"id":5,"name":"Africa"},
                    {"id":6,"name":"Asia"},
                    {"id":7,"name":"Oceania"}
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
    async fn day18_test_total() {
        let app = router().await;

        let client = TestClient::new(app);

        // reset the database
        let res = client.post("/18/reset").send().await;
        assert_eq!(res.status(), StatusCode::OK);

        // create orders
        let res = client
            .post("/18/orders")
            .body(
                json!(
                [
                    {"id":1,"region_id":2,"gift_name":"Board Game","quantity":5},
                    {"id":2,"region_id":2,"gift_name":"Origami Set","quantity":8},
                    {"id":3,"region_id":3,"gift_name":"Action Figure","quantity":12},
                    {"id":4,"region_id":4,"gift_name":"Teddy Bear","quantity":10},
                    {"id":5,"region_id":2,"gift_name":"Yarn Ball","quantity":6},
                    {"id":6,"region_id":3,"gift_name":"Art Set","quantity":3},
                    {"id":7,"region_id":5,"gift_name":"Robot Lego Kit","quantity":5},
                    {"id":8,"region_id":6,"gift_name":"Drone","quantity":9}
                  ]
                        )
                .to_string(),
            )
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);

        // create regions
        let res = client
            .post("/18/regions")
            .body(
                json!(
                [
                    {"id":1,"name":"North Pole"},
                    {"id":2,"name":"Europe"},
                    {"id":3,"name":"North America"},
                    {"id":4,"name":"South America"},
                    {"id":5,"name":"Africa"},
                    {"id":6,"name":"Asia"},
                    {"id":7,"name":"Oceania"}
                  ]
                        )
                .to_string(),
            )
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);

        // test total
        let res = client.get("/18/regions/total").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.json::<Value>().await,
            json!([
              {"region":"Africa","total":5},
              {"region":"Asia","total":9},
              {"region":"Europe","total":19},
              {"region":"North America","total":15},
              {"region":"South America","total":10}
            ])
        );
    }

    #[tokio::test]
    async fn day18_test_best_region() {
        let app = router().await;

        let client = TestClient::new(app);

        // reset the database
        let res = client.post("/18/reset").send().await;
        assert_eq!(res.status(), StatusCode::OK);

        // create orders
        let res = client
            .post("/18/orders")
            .body(
                json!(
                [
                    {"id":1,"region_id":2,"gift_name":"Toy Train","quantity":5},
                    {"id":2,"region_id":2,"gift_name":"Toy Train","quantity":3},
                    {"id":3,"region_id":2,"gift_name":"Doll","quantity":8},
                    {"id":4,"region_id":3,"gift_name":"Toy Train","quantity":3},
                    {"id":5,"region_id":2,"gift_name":"Teddy Bear","quantity":6},
                    {"id":6,"region_id":3,"gift_name":"Action Figure","quantity":12},
                    {"id":7,"region_id":4,"gift_name":"Board Game","quantity":10},
                    {"id":8,"region_id":3,"gift_name":"Teddy Bear","quantity":1},
                    {"id":9,"region_id":3,"gift_name":"Teddy Bear","quantity":2}
                ])
                .to_string(),
            )
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);

        // create regions
        let res = client
            .post("/18/regions")
            .body(
                json!(
                [
                    {"id":1,"name":"North Pole"},
                    {"id":2,"name":"South Pole"},
                    {"id":3,"name":"Kiribati"},
                    {"id":4,"name":"Baker Island"}
                  ])
                .to_string(),
            )
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);

        let res = client.get("/18/regions/top_list/2").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.json::<Value>().await,
            json!([
              {"region":"Baker Island","top_gifts":["Board Game"]},
              {"region":"Kiribati","top_gifts":["Action Figure","Teddy Bear"]},
              {"region":"North Pole","top_gifts":[]},
              {"region":"South Pole","top_gifts":["Doll","Toy Train"]}
            ])
        );
    }
}
