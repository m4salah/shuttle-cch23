use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Instant,
};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Datelike, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::convert::Into;
use ulid::Ulid;
use uuid::Uuid;

type SharedState = Arc<RwLock<PacketState>>;

#[derive(Default, Clone)]
struct PacketState {
    packet_saved_at: HashMap<String, Instant>,
}
pub fn router() -> Router {
    let packet_state = Arc::new(RwLock::new(PacketState {
        packet_saved_at: HashMap::new(),
    }));

    Router::new()
        .route("/12/health", get(|| async { StatusCode::OK }))
        .route("/12/save/:packet_id", post(save_packet_id))
        .route("/12/load/:packet_id", get(load_packet_id))
        .route("/12/ulids", post(ulids_to_uuids))
        .route("/12/ulids/:weekday", post(ulids_weekday))
        .with_state(packet_state)
}

async fn save_packet_id(
    State(packet_extension): State<SharedState>,
    Path(packet_id): Path<String>,
) -> Result<(), StatusCode> {
    packet_extension
        .write()
        .map_err(|e| {
            tracing::error!("error while getting write lock {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .packet_saved_at
        .insert(packet_id, Instant::now());
    Ok(())
}

async fn load_packet_id(
    State(packet_extension): State<SharedState>,
    Path(packet_id): Path<String>,
) -> Result<String, StatusCode> {
    let db = &packet_extension
        .read()
        .map_err(|e| {
            tracing::error!("error while getting read lock {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .packet_saved_at;

    if let Some(value) = db.get(&packet_id) {
        let duration_since_saved = value.elapsed().as_secs();
        return Ok(format!("{duration_since_saved}"));
    }
    Err(StatusCode::NOT_FOUND)
}

async fn ulids_to_uuids(Json(ulids): Json<Vec<String>>) -> Json<Vec<String>> {
    // Convert all the ULIDs to UUIDs
    let uuids: Vec<String> = ulids
        .iter()
        .filter_map(|ulid| {
            if let Ok(ulid) = Ulid::from_string(ulid) {
                let uuid: Uuid = ulid.into();
                Some(uuid.to_string())
            } else {
                None
            }
        })
        .rev()
        .collect();
    Json(uuids)
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UlidsWeekdayResult {
    #[serde(rename = "christmas eve")]
    pub christmas_eve: usize,
    pub weekday: usize,
    #[serde(rename = "in the future")]
    pub in_the_future: usize,
    #[serde(rename = "LSB is 1")]
    pub lsb_is_1: usize,
}

async fn ulids_weekday(
    Path(weekday): Path<u8>,
    Json(ulids): Json<Vec<String>>,
) -> Result<Json<UlidsWeekdayResult>, StatusCode> {
    let weekday = Weekday::try_from(weekday).map_err(|e| {
        tracing::error!("Failed to parse weekday  {e}");
        StatusCode::BAD_REQUEST
    })?;
    // Convert all the ULIDs to UUIDs
    let dates: Vec<DateTime<Utc>> = ulids
        .iter()
        .filter_map(|ulid| {
            if let Ok(ulid) = Ulid::from_string(ulid) {
                let day: DateTime<Utc> = ulid.datetime().into();
                Some(day)
            } else {
                None
            }
        })
        .collect();

    Ok(Json(UlidsWeekdayResult {
        christmas_eve: dates
            .iter()
            .filter(|date| date.day() == 24 && date.month() == 12)
            .count(),
        weekday: dates
            .iter()
            .filter(|date| date.weekday() == weekday)
            .count(),
        in_the_future: dates.iter().filter(|date| date > &&Utc::now()).count(),
        lsb_is_1: ulids
            .iter()
            .map(|ulid| Ulid::from_string(ulid).unwrap())
            .filter(|ulid| ulid.0 & 1 == 1)
            .count(),
    }))
}
