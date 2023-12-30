use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use super::day13::{create_orders, SqlState};

pub fn router(pool: Pool<Postgres>) -> Router {
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
    Ok(Json(regions))
}
