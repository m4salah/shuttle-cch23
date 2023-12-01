use std::str::FromStr;

use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Router};

async fn hello_world() -> impl IntoResponse {
    "Hello, World!"
}

async fn internal_server_error() -> impl IntoResponse {
    StatusCode::INTERNAL_SERVER_ERROR
}

async fn packet_ids(Path(ids): Path<String>) -> impl IntoResponse {
    let packet_ids: Vec<i32> = ids
        .split('/')
        .map(|id_str| i32::from_str(id_str).unwrap())
        .collect();
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

#[allow(dead_code)]
fn app() -> Router {
    Router::new()
        .route("/", get(hello_world))
        .route("/-1/error", get(internal_server_error))
        .route("/1/*ids", get(packet_ids))
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    Ok(app().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test_helper::TestClient;

    #[tokio::test]
    async fn hello_world() {
        let app = app();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let client = TestClient::new(app);
        let res = client.get("/").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await, "Hello, World!");
    }

    #[tokio::test]
    async fn internal_server_error() {
        let app = app();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let client = TestClient::new(app);
        let res = client.get("/-1/error").send().await;
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(res.text().await, "");
    }

    #[tokio::test]
    async fn num1_xor_num2_pow_3() {
        let app = app();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let client = TestClient::new(app);
        let res = client.get("/1/3/5").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = ((3 ^ 5) as i32).pow(3);
        assert_eq!(res.text().await, format!("{expected}"));
    }

    #[tokio::test]
    async fn one_packet_ids() {
        let app = app();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let client = TestClient::new(app);
        let res = client.get("/1/10").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = 1000;
        assert_eq!(res.text().await, format!("{expected}"));
    }

    #[tokio::test]
    async fn multi_packet_ids() {
        let app = app();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let client = TestClient::new(app);
        let res = client.get("/1/4/5/8/10").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = 27;
        assert_eq!(res.text().await, format!("{expected}"));
    }

    #[tokio::test]
    async fn multi_packet_ids_more_than_20_ids() {
        let app = app();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let client = TestClient::new(app);
        let res = client
            .get("/1/1/2/3/4/5/6/7/8/9/10/11/12/13/14/15/16/17/18/19/20/21")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let expected = "packet ids must be between 1 and 20 inclusive packets in a sled";
        assert_eq!(res.text().await, format!("{expected}"));
    }
}
