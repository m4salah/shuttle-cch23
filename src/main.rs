use std::str::FromStr;

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

async fn hello_world() -> impl IntoResponse {
    "Hello, World!"
}

async fn internal_server_error() -> impl IntoResponse {
    StatusCode::INTERNAL_SERVER_ERROR
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

#[derive(Debug, Serialize, Deserialize)]
struct Reindeer {
    name: String,
    strength: i32,
}
async fn sum_strength(Json(reindeers): Json<Vec<Reindeer>>) -> impl IntoResponse {
    reindeers
        .iter()
        .fold(0, |acc, r| acc + r.strength)
        .to_string()
}

#[derive(Debug, Serialize, Deserialize)]
struct ContestReindeer {
    name: String,
    strength: i32,
    speed: f32,
    height: u32,
    antler_width: u32,
    snow_magic_power: i64,
    favorite_food: String,
    #[serde(alias = "cAnD13s_3ATeN-yesT3rdAy")]
    candies: i32,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
struct ContestResult {
    fastest: String,
    tallest: String,
    magician: String,
    consumer: String,
}
async fn contest(Json(reindeers): Json<Vec<ContestReindeer>>) -> impl IntoResponse {
    let fastest = reindeers.iter().max_by(|a, b| a.strength.cmp(&b.strength));
    let tallest = reindeers.iter().max_by(|a, b| a.height.cmp(&b.height));
    let magician = reindeers
        .iter()
        .max_by(|a, b| a.snow_magic_power.cmp(&b.snow_magic_power));
    let candiest = reindeers.iter().max_by(|a, b| a.candies.cmp(&b.candies));

    match (fastest, tallest, magician, candiest) {
        (Some(f), Some(t), Some(m), Some(c)) => Json(ContestResult {
            fastest: format!(
                "Speeding past the finish line with a strength of {} is {}",
                f.strength, f.name
            ),
            tallest: format!(
                "{} is standing tall with his {} cm wide antlers",
                t.name, t.antler_width
            ),
            magician: format!(
                "{} could blast you away with a snow magic power of {}",
                m.name, m.snow_magic_power
            ),
            consumer: format!("{} ate lots of candies, but also some grass", c.name),
        })
        .into_response(),
        _ => (StatusCode::BAD_REQUEST, "Invalid contest").into_response(),
    }
}

#[allow(dead_code)]
fn app() -> Router {
    Router::new()
        .route("/", get(hello_world))
        .route("/-1/error", get(internal_server_error))
        .route("/1/*ids", get(packet_ids))
        .route("/4/strength", post(sum_strength))
        .route("/4/contest", post(contest))
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    Ok(app().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test_helper::TestClient;
    use serde_json::json;

    #[tokio::test]
    async fn hello_world() {
        let app = app();

        let client = TestClient::new(app);
        let res = client.get("/").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await, "Hello, World!");
    }

    #[tokio::test]
    async fn internal_server_error() {
        let app = app();

        let client = TestClient::new(app);
        let res = client.get("/-1/error").send().await;
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(res.text().await, "");
    }

    #[tokio::test]
    async fn num1_xor_num2_pow_3() {
        let app = app();

        let client = TestClient::new(app);
        let res = client.get("/1/3/5").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = ((3 ^ 5) as i32).pow(3);
        assert_eq!(res.text().await, format!("{expected}"));
    }

    #[tokio::test]
    async fn one_packet_ids() {
        let app = app();

        let client = TestClient::new(app);
        let res = client.get("/1/10").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = 1000;
        assert_eq!(res.text().await, format!("{expected}"));
    }

    #[tokio::test]
    async fn multi_packet_ids() {
        let app = app();

        let client = TestClient::new(app);
        let res = client.get("/1/4/5/8/10").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = 27;
        assert_eq!(res.text().await, format!("{expected}"));
    }

    #[tokio::test]
    async fn multi_packet_ids_more_than_20_ids() {
        let app = app();

        let client = TestClient::new(app);
        let res = client
            .get("/1/1/2/3/4/5/6/7/8/9/10/11/12/13/14/15/16/17/18/19/20/21")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let expected = "packet ids must be between 1 and 20 inclusive packets in a sled";
        assert_eq!(res.text().await, format!("{expected}"));
    }

    #[tokio::test]
    async fn sum_of_strength() {
        let app = app();

        let client = TestClient::new(app);
        let res = client
            .post("/4/strength")
            .body(
                json!([
                  { "name": "Dasher", "strength": 5 },
                  { "name": "Dancer", "strength": 6 },
                  { "name": "Prancer", "strength": 4 },
                  { "name": "Vixen", "strength": 7 }
                ])
                .to_string(),
            )
            .header("Content-Type", "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = "22";
        assert_eq!(res.text().await, format!("{expected}"));
    }

    #[tokio::test]
    async fn valid_contest() {
        let app = app();

        let client = TestClient::new(app);
        let res = client
            .post("/4/contest")
            .body(
                json!([
                {
                      "name": "Dasher",
                      "strength": 5,
                      "speed": 50.4,
                      "height": 80,
                      "antler_width": 36,
                      "snow_magic_power": 9001,
                      "favorite_food": "hay",
                      "cAnD13s_3ATeN-yesT3rdAy": 2
                    },
                    {
                      "name": "Dancer",
                      "strength": 6,
                      "speed": 48.2,
                      "height": 65,
                      "antler_width": 37,
                      "snow_magic_power": 4004,
                      "favorite_food": "grass",
                      "cAnD13s_3ATeN-yesT3rdAy": 5
                    }
                                ])
                .to_string(),
            )
            .header("Content-Type", "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = ContestResult {
            fastest: "Speeding past the finish line with a strength of 6 is Dancer".to_string(),
            tallest: "Dasher is standing tall with his 36 cm wide antlers".to_string(),
            magician: "Dasher could blast you away with a snow magic power of 9001".to_string(),
            consumer: "Dancer ate lots of candies, but also some grass".to_string(),
        };
        assert_eq!(res.json::<ContestResult>().await, expected);
    }

    #[tokio::test]
    async fn invalid_contest() {
        let app = app();

        let client = TestClient::new(app);
        let res = client
            .post("/4/contest")
            .body(json!([]).to_string())
            .header("Content-Type", "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}
