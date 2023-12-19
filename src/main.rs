use std::env;

use axum::Router;
use sqlx::{PgPool, Pool, Postgres};

mod handlers;
fn app(pool: Pool<Postgres>) -> Router {
    handlers::router(pool)
}

#[shuttle_runtime::main]
async fn shuttle_service() -> shuttle_axum::ShuttleAxum {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set the env variable");
    let pool = PgPool::connect(&database_url).await.unwrap();
    Ok(app(pool).into())
}
