use std::{error::Error, thread, time::Duration};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use dms_coordinates::DMS;
use s2::{cell::Cell, cellid::CellID};
use serde::Deserialize;

pub fn router(geocoding_api_key: String) -> Router {
    Router::new()
        .route("/21/health", get(|| async { StatusCode::OK }))
        .route("/21/coords/:binary", get(coords))
        .route("/21/country/:binary", get(country))
        .with_state(geocoding_api_key)
}

async fn coords(Path(binary): Path<String>) -> Result<String, StatusCode> {
    let b = u64::from_str_radix(binary.as_str(), 2).map_err(|e| {
        tracing::error!("error converting binary to u64 {e}");
        StatusCode::BAD_REQUEST
    })?;
    let cell_id = CellID(b);
    let center = Cell::from(cell_id).center();
    let (lat, long) = (
        DMS::from_decimal_degrees(center.latitude().deg(), true),
        DMS::from_decimal_degrees(center.longitude().deg(), false),
    );

    tracing::info!(
        "lat: {}, long: {}",
        center.latitude().deg(),
        center.longitude().deg(),
    );
    Ok(format!(
        "{}°{}'{:.3}''{} {}°{}'{:.3}''{}",
        lat.degrees,
        lat.minutes,
        lat.seconds,
        lat.bearing,
        long.degrees,
        long.minutes,
        long.seconds,
        long.bearing
    ))
}

async fn country(
    Path(binary): Path<String>,
    State(geocoding_api_key): State<String>,
) -> Result<String, StatusCode> {
    let b = u64::from_str_radix(binary.as_str(), 2).map_err(|e| {
        tracing::error!("error converting binary to u64 {e}");
        StatusCode::BAD_REQUEST
    })?;
    let cell_id = CellID(b);
    let center = Cell::from(cell_id).center();

    fetch_country_from_latlong(
        geocoding_api_key,
        center.latitude().deg(),
        center.longitude().deg(),
    )
    .await
    .map_err(|e| {
        tracing::error!("error while fetching country {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(Deserialize)]
struct Geocode {
    pub address: Address,
}

#[derive(Deserialize)]
pub struct Address {
    pub country: String,
}

async fn fetch_country_from_latlong(
    geocode_api_key: String,
    lat: f64,
    long: f64,
) -> Result<String, Box<dyn Error>> {
    let endpoint =
        format!("https://geocode.maps.co/reverse?lat={lat}&lon={long}&api_key={geocode_api_key}");
    let country = loop {
        let endpoint = endpoint.clone();
        if let Ok(req) = reqwest::get(endpoint).await {
            if let Ok(c) = req.json::<Geocode>().await {
                break c.address.country;
            }
        }
        // sleep for 1 second and repeat, because the rate limit in this service is 1 req/second
        // https://geocode.maps.co/
        thread::sleep(Duration::from_secs(1));
    };
    Ok(country)
}
