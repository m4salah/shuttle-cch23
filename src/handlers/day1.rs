use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get};
use std::str::FromStr;

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/1/*ids", get(packet_ids))
        .route("/1/health", get(|| async { StatusCode::OK }))
}

async fn packet_ids(Path(ids): Path<String>) -> impl IntoResponse {
    let packet_ids: Vec<i32> = ids
        .split('/')
        // TODO: How to handle this gracefully?
        .map(|id_str| i32::from_str(id_str).unwrap_or(0))
        .collect();

    // validate on the length of the ids
    if packet_ids.len() > 20 {
        return (
            StatusCode::BAD_REQUEST,
            "packet ids must be between 1 and 20 inclusive packets in a sled",
        )
            .into_response();
    }
    let result = packet_ids.iter().fold(0, |acc, prev| acc ^ prev).pow(3);
    format!("{result}").into_response()
}
