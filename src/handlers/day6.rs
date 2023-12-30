use axum::{
    http::StatusCode,
    routing::{get, post},
    Json,
};
use fancy_regex::Regex;
use serde::{Deserialize, Serialize};

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/6", post(elf_on_shelf))
        .route("/6/health", get(|| async { StatusCode::OK }))
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
struct ElfOnShelfResult {
    elf: u64,
    #[serde(rename = "elf on a shelf")]
    elf_on_a_shelf: u64,
    #[serde(rename = "shelf with no elf on it")]
    shelf_with_no_elf_on_it: u64,
}

async fn elf_on_shelf(elf_text: String) -> Result<Json<ElfOnShelfResult>, StatusCode> {
    tracing::info!("elf_text: {elf_text}");
    let shelf = Regex::new("shelf")
        .map_err(|e| {
            eprintln!("ERR: coud't make the regex {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .captures_iter(elf_text.as_str())
        .count() as u64;
    let elf_on_a_shelf = Regex::new("elf(?= on a shelf)")
        .map_err(|e| {
            eprintln!("ERR: coud't make the regex {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .captures_iter(elf_text.as_str())
        .count() as u64;

    let elf = Regex::new("elf")
        .map_err(|e| {
            eprintln!("ERR: coud't make the regex {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .captures_iter(elf_text.as_str())
        .count() as u64;

    tracing::info!(
        "elf_text result: {:?}",
        ElfOnShelfResult {
            elf,
            elf_on_a_shelf,
            shelf_with_no_elf_on_it: shelf - elf_on_a_shelf,
        }
    );
    Ok(Json(ElfOnShelfResult {
        elf,
        elf_on_a_shelf,
        shelf_with_no_elf_on_it: shelf - elf_on_a_shelf,
    }))
}
