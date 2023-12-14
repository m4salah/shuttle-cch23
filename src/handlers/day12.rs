use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Instant, SystemTime},
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

async fn ulids_to_uuids(Json(ulids): Json<Vec<String>>) -> Result<Json<Vec<String>>, StatusCode> {
    // Convert all the ULIDs to UUIDs
    let mut uuids = ulids
        .iter()
        .map(|ulid| {
            let ulid = Ulid::from_string(&ulid).map_err(|e| {
                tracing::error!("Failed to parse ULID {ulid}: {e}");
                StatusCode::BAD_REQUEST
            })?;
            let uuid: Uuid = ulid.into();
            Ok(uuid.to_string())
        })
        .collect::<Result<Vec<String>, StatusCode>>()?;
    uuids.reverse();
    Ok(Json(uuids))
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
    let dates = ulids
        .iter()
        .map(|ulid| {
            let ulid = Ulid::from_string(&ulid).map_err(|e| {
                tracing::error!("Failed to parse ULID {ulid}: {e}");
                StatusCode::BAD_REQUEST
            })?;
            let day: DateTime<Utc> = ulid.datetime().into();
            Ok(day)
        })
        .collect::<Result<Vec<DateTime<Utc>>, StatusCode>>()?;

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
            .map(|ulid| Ulid::from_string(&ulid).unwrap())
            .filter(|ulid| ulid.0 & 1 == 1)
            .count(),
    }))
}

#[cfg(test)]
mod tests {

    use std::{thread, time::Duration};

    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;
    use reqwest::header::CONTENT_TYPE;
    use serde_json::{json, Value};

    #[tokio::test]
    async fn day12_health() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/12/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day12_save_and_load() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.post("/12/save/helloWorld").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        thread::sleep(Duration::from_secs(5));
        let res = client.get("/12/load/helloWorld").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await, "5");
    }

    #[tokio::test]
    async fn day12_ulids() {
        let app = router();

        let client = TestClient::new(app);
        let res = client
            .post("/12/ulids")
            .body(
                json!([
                    "01BJQ0E1C3Z56ABCD0E11HYX4M",
                    "01BJQ0E1C3Z56ABCD0E11HYX5N",
                    "01BJQ0E1C3Z56ABCD0E11HYX6Q",
                    "01BJQ0E1C3Z56ABCD0E11HYX7R",
                    "01BJQ0E1C3Z56ABCD0E11HYX8P"
                ])
                .to_string(),
            )
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::OK);
        let expected = json!([
            "015cae07-0583-f94c-a5b1-a070431f7516",
            "015cae07-0583-f94c-a5b1-a070431f74f8",
            "015cae07-0583-f94c-a5b1-a070431f74d7",
            "015cae07-0583-f94c-a5b1-a070431f74b5",
            "015cae07-0583-f94c-a5b1-a070431f7494",
        ]);

        assert_eq!(res.json::<Value>().await, expected);
    }

    #[tokio::test]
    async fn day12_ulids_weekday() {
        let app = router();

        let client = TestClient::new(app);
        let res = client
            .post("/12/ulids/5")
            .body(
                json!([
                    "00WEGGF0G0J5HEYXS3D7RWZGV8",
                    "76EP4G39R8JD1N8AQNYDVJBRCF",
                    "018CJ7KMG0051CDCS3B7BFJ3AK",
                    "00Y986KPG0AMGB78RD45E9109K",
                    "010451HTG0NYWMPWCEXG6AJ8F2",
                    "01HH9SJEG0KY16H81S3N1BMXM4",
                    "01HH9SJEG0P9M22Z9VGHH9C8CX",
                    "017F8YY0G0NQA16HHC2QT5JD6X",
                    "03QCPC7P003V1NND3B3QJW72QJ"
                ])
                .to_string(),
            )
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::OK);

        let expected = UlidsWeekdayResult {
            christmas_eve: 3,
            weekday: 1,
            in_the_future: 2,
            lsb_is_1: 5,
        };

        assert_eq!(res.json::<UlidsWeekdayResult>().await, expected);
    }
}
