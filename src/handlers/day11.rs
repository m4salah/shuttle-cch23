use axum::{
    extract::Multipart,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use image::{GenericImageView, Rgba};
use tower_http::services::ServeDir;

pub fn router() -> Router {
    Router::new()
        .route("/11/health", get(|| async { StatusCode::OK }))
        .route("/11/red_pixels", post(red_pixels))
        .nest_service("/11/assets", ServeDir::new("assets"))
}

async fn red_pixels(mut multipart: Multipart) -> Result<String, StatusCode> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("");
        let content_type = field.content_type().unwrap_or("");

        if name == "image" && content_type == "image/png" {
            let data = field
                .bytes()
                .await
                .map_err(|e| {
                    tracing::error!("error getting bytes of the image {e}");
                    StatusCode::BAD_REQUEST
                })?
                .to_vec();
            let red_pixels = image::load_from_memory(data.as_slice())
                .map_err(|e| {
                    tracing::error!("error while loading image {e}");
                    StatusCode::BAD_REQUEST
                })?
                .pixels()
                .fold(0, |red_pixels, (_, _, Rgba([r, g, b, _]))| {
                    if r as u32 > g as u32 + b as u32 {
                        red_pixels + 1
                    } else {
                        red_pixels
                    }
                });
            return Ok(format!("{red_pixels}"));
        }
    }
    Ok("0".to_string())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use axum::{body::Empty, http::StatusCode};
    use axum_test_helper::TestClient;
    use reqwest::multipart::{Form, Part};

    #[tokio::test]
    async fn day11_health() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/11/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day11_right_content_type_and_content_length() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/11/assets/decoration.png").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers().get("content-length").unwrap(), "787297");
        assert_eq!(res.headers().get("content-type").unwrap(), "image/png");
    }
}
