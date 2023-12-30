use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json,
};
use serde::{Deserialize, Serialize};

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/4/strength", post(sum_strength))
        .route("/4/contest", post(contest))
        .route("/4/health", get(|| async { StatusCode::OK }))
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
    strength: u32,
    speed: f32,
    height: u32,
    antler_width: u32,
    snow_magic_power: u32,
    favorite_food: String,
    #[serde(alias = "cAnD13s_3ATeN-yesT3rdAy")]
    candies: u32,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
struct ContestResult {
    fastest: String,
    tallest: String,
    magician: String,
    consumer: String,
}
async fn contest(Json(reindeers): Json<Vec<ContestReindeer>>) -> impl IntoResponse {
    let fastest = reindeers.iter().max_by(|a, b| a.speed.total_cmp(&b.speed));
    let tallest = reindeers.iter().max_by_key(|r| r.height);
    let magician = reindeers.iter().max_by_key(|r| r.snow_magic_power);
    let candiest = reindeers.iter().max_by_key(|r| r.candies);

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
            consumer: format!(
                "{} ate lots of candies, but also some {}",
                c.name, c.favorite_food
            ),
        })
        .into_response(),
        _ => (StatusCode::BAD_REQUEST, "Invalid contest").into_response(),
    }
}
