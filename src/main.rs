use axum::Router;

mod handlers;
#[allow(dead_code)]
async fn app() -> Router {
    handlers::router().await
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    dotenv::dotenv().ok();
    Ok(app().await.into())
}
