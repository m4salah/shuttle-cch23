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
pub struct ContestReindeer {
    pub name: String,
    pub strength: i32,
    pub speed: f32,
    pub height: u32,
    pub antler_width: u32,
    pub snow_magic_power: i64,
    pub favorite_food: String,
    #[serde(alias = "cAnD13s_3ATeN-yesT3rdAy")]
    pub candies: i32,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ContestResult {
    pub fastest: String,
    pub tallest: String,
    pub magician: String,
    pub consumer: String,
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
