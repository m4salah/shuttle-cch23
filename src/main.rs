use axum::Router;

mod handlers;
#[allow(dead_code)]
fn app() -> Router {
    handlers::router()
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    Ok(app().into())
}
