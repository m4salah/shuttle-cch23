use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
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
) -> Result<Response, Json<Vec<String>>> {
    println!("{pagination:?} {names:?}");

    match (pagination.split, pagination.limit) {
        (None, None) => {
            Ok(Json(names.get(pagination.offset..).ok_or(Json(vec![]))?.to_vec()).into_response())
        }

        (None, Some(limit)) => Ok(Json(
            names
                .get(pagination.offset..pagination.offset + limit)
                .ok_or(Json(vec![]))?
                .to_vec(),
        )
        .into_response()),

        (Some(split), None) => Ok(Json(
            names
                .chunks(split)
                .map(|s| s.into())
                .collect::<Vec<Vec<String>>>(),
        )
        .into_response()),

        (Some(split), Some(limit)) => Ok(Json(
            names
                .get(pagination.offset..pagination.offset + limit)
                .ok_or(Json(vec![]))?
                .chunks(split)
                .map(|s| s.into())
                .collect::<Vec<Vec<String>>>(),
        )
        .into_response()),
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
