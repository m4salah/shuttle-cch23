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
