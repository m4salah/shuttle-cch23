use axum::{routing::post, Json};
use fancy_regex::Regex;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

pub fn router() -> axum::Router {
    axum::Router::new().route("/6", post(elf_on_shelf))
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
    let shelf_with_no_elf_on_it = Regex::new("(?<!elf on a )shelf")
        .map_err(|e| {
            eprintln!("ERR: coud't make the regex {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .captures_iter(elf_text.as_str())
        .count() as u64;
    let elf_on_a_shelf = elf_text.matches("elf on a shelf").count() as u64;
    let elf = elf_text.matches("elf").count() as u64;

    Ok(Json(ElfOnShelfResult {
        elf,
        elf_on_a_shelf,
        shelf_with_no_elf_on_it,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;

    #[tokio::test]
    async fn test_empty_shelves() {
        let app = router();

        let client = TestClient::new(app);
        let res = client
            .post("/6")
            .body("")
            .header("Content-Type", "text/plain")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = ElfOnShelfResult {
            ..Default::default()
        };
        assert_eq!(res.json::<ElfOnShelfResult>().await, expected);
    }

    #[tokio::test]
    async fn test_no_elf_on_shelves() {
        let app = router();

        let client = TestClient::new(app);
        let res = client
            .post("/6")
            .body("there is a shelf. another shelf here.")
            .header("Content-Type", "text/plain")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = ElfOnShelfResult {
            elf: 2,
            shelf_with_no_elf_on_it: 2,
            ..Default::default()
        };
        assert_eq!(res.json::<ElfOnShelfResult>().await, expected);
    }

    #[tokio::test]
    async fn test_mixed_shelves() {
        let app = router();

        let client = TestClient::new(app);
        let res = client
            .post("/6")
            .body("there is an elf on a shelf. another shelf here. elf on a shelf.")
            .header("Content-Type", "text/plain")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = ElfOnShelfResult {
            elf: 5,
            elf_on_a_shelf: 2,
            shelf_with_no_elf_on_it: 1,
        };
        assert_eq!(res.json::<ElfOnShelfResult>().await, expected);
    }

    #[tokio::test]
    async fn elf() {
        let app = router();

        let client = TestClient::new(app);
        let res = client
            .post("/6")
            .body(
                "The mischievous elf peeked out from behind the toy workshop,\
      and another elf joined in the festive dance.\
      Look, there is also an elf on that shelf!",
            )
            .header("Content-Type", "text/plain")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = ElfOnShelfResult {
            elf: 4,
            ..Default::default()
        };
        assert_eq!(res.json::<ElfOnShelfResult>().await.elf, expected.elf);
    }
    #[tokio::test]
    async fn elf_on_shelf() {
        let app = router();

        let client = TestClient::new(app);
        let res = client
            .post("/6")
            .body(
                "there is an elf on a shelf on an elf.\
      there is also another shelf in Belfast.",
            )
            .header("Content-Type", "text/plain")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = ElfOnShelfResult {
            elf: 5,
            shelf_with_no_elf_on_it: 1,
            elf_on_a_shelf: 1,
        };
        assert_eq!(res.json::<ElfOnShelfResult>().await, expected);
    }
}
