use axum::{response::IntoResponse, routing::post, Json};
use serde::{Deserialize, Serialize};

pub fn router() -> axum::Router {
    axum::Router::new().route("/6", post(elf_on_shelf))
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
struct ElfOnShelfResult {
    elf: u32,
    #[serde(alias = "elf on a shelf")]
    elf_on_shelf: u32,
    #[serde(alias = "shelf with no elf on it")]
    shelf_with_no_elf: u32,
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;

    #[tokio::test]
    async fn elf() {
        let app = router();

        let client = TestClient::new(app);
        let res = client
            .post("/6")
            .body(
                "The mischievous elf peeked out from behind the toy workshop,
      and another elf joined in the festive dance.
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
            .body("there is an elf on a shelf on an elf. there is also another shelf in Belfast.")
            .header("Content-Type", "text/plain")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = ElfOnShelfResult {
            elf: 5,
            shelf_with_no_elf: 1,
            elf_on_shelf: 1,
        };
        assert_eq!(res.json::<ElfOnShelfResult>().await, expected);
    }
}
