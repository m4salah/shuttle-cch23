use std::collections::HashSet;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

pub fn router() -> Router {
    Router::new()
        .route("/22/health", get(|| async { StatusCode::OK }))
        .route("/22/integers", post(integers))
        .route("/22/rocket", post(rocket))
}

async fn integers(content: String) -> impl IntoResponse {
    let mut result: HashSet<usize> = HashSet::new();
    for line in content.lines() {
        if let Ok(num) = line.trim().parse::<usize>() {
            if result.take(&num).is_none() {
                result.insert(num);
            }
        }
    }
    let rep = result.into_iter().next().unwrap_or_default();
    "🎁".repeat(rep)
}

async fn rocket(content: String) -> impl IntoResponse {
    let mut result: HashSet<usize> = HashSet::new();
    for line in content.lines() {
        if let Ok(num) = line.trim().parse::<usize>() {
            if result.take(&num).is_none() {
                result.insert(num);
            }
        }
    }
    let gifts_count = result.into_iter().next().unwrap_or_default();
    "🎁".repeat(gifts_count)
}
