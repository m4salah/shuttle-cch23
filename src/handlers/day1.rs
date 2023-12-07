use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get};
use std::str::FromStr;

pub fn router() -> axum::Router {
    axum::Router::new().route("/1/*ids", get(packet_ids))
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;

    #[tokio::test]
    async fn num1_xor_num2_pow_3() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/1/3/5").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = ((3 ^ 5) as i32).pow(3);
        assert_eq!(res.text().await, format!("{expected}"));
    }

    #[tokio::test]
    async fn one_packet_ids() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/1/10").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = 1000;
        assert_eq!(res.text().await, format!("{expected}"));
    }
}
