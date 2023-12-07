use axum::{http::StatusCode, response::IntoResponse, routing::post, Json};
use serde::{Deserialize, Serialize};

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/4/strength", post(sum_strength))
        .route("/4/contest", post(contest))
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;
    use serde_json::json;

    #[tokio::test]
    async fn sum_of_strength() {
        let app = router();
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
        let app = router();

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
        let app = router();

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
