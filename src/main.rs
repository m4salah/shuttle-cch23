use axum::Router;

mod handlers;
async fn app() -> Router {
    handlers::router().await
}

#[shuttle_runtime::main]
async fn shuttle_service() -> shuttle_axum::ShuttleAxum {
    dotenv::dotenv().ok();
    Ok(app().await.into())
}
