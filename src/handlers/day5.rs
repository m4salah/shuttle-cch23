use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json,
};
use serde::{Deserialize, Serialize};

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/5/health", get(|| async { StatusCode::OK }))
        .route("/5", post(slicing))
}

#[derive(Debug, Serialize, Deserialize)]
struct PaginationQuery {
    #[serde(default)]
    split: Option<usize>,

    #[serde(default)]
    offset: usize,

    #[serde(default)]
    limit: Option<usize>,
}

async fn slicing(
    Query(pagination): Query<PaginationQuery>,
    Json(names): Json<Vec<String>>,
) -> impl IntoResponse {
    println!("{pagination:?} {names:?}");

    match (pagination.split, pagination.limit) {
        (None, None) => {
            Json(names.get(pagination.offset..).unwrap_or_default().to_vec()).into_response()
        }

        (None, Some(limit)) => Json(
            names
                .get(pagination.offset..pagination.offset + limit)
                .unwrap_or_default()
                .to_vec(),
        )
        .into_response(),

        (Some(split), None) => Json(
            names
                .chunks(split)
                .map(|s| s.into())
                .collect::<Vec<Vec<String>>>(),
        )
        .into_response(),

        (Some(split), Some(limit)) => Json(
            names
                .get(pagination.offset..pagination.offset + limit)
                .unwrap_or_default()
                .chunks(split)
                .map(|s| s.into())
                .collect::<Vec<Vec<String>>>(),
        )
        .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;

    #[tokio::test]
    async fn day5_health() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/5/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }
}
