use axum::{response::IntoResponse, routing::post, Json};
use serde::{Deserialize, Serialize};

pub fn router() -> axum::Router {
    axum::Router::new().route("/6", post(elf_on_shelf))
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ElfOnShelfResult {
    pub elf: u32,
    #[serde(alias = "elf on a shelf")]
    pub elf_on_shelf: u32,
    #[serde(alias = "shelf with no elf on it")]
    pub shelf_with_no_elf: u32,
}

async fn elf_on_shelf(elf_text: String) -> impl IntoResponse {
    let shelf = elf_text.matches("shelf").count() as u32;
    let elf_on_shelf = elf_text.matches("elf on a shelf").count() as u32;
    let elf = elf_text.matches("elf").count() as u32;
    Json(ElfOnShelfResult {
        elf,
        elf_on_shelf,
        shelf_with_no_elf: shelf - elf_on_shelf,
    })
}
