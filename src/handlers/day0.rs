use axum::{http::StatusCode, response::IntoResponse, routing::get};

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/", get(hello_world))
        .route("/-1/error", get(internal_server_error))
}

pub async fn internal_server_error() -> impl IntoResponse {
    StatusCode::INTERNAL_SERVER_ERROR
}

async fn hello_world() -> impl IntoResponse {
    "Hello, World!"
}
