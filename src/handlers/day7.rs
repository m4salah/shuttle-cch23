use std::collections::HashMap;

use axum::{http::StatusCode, routing::get, Json};
use axum_extra::{headers::Cookie, TypedHeader};
use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/7/decode", get(santa_cookie))
        .route("/7/bake", get(secret_cookie))
        .route("/7/health", get(|| async { StatusCode::OK }))
}

#[axum::debug_handler]
async fn santa_cookie(TypedHeader(cookie): TypedHeader<Cookie>) -> Result<Json<Value>, StatusCode> {
    let recipe = cookie.get("recipe").ok_or(StatusCode::BAD_REQUEST)?;

    let de = general_purpose::STANDARD.decode(recipe).map_err(|e| {
        eprintln!("ERR: error while decoding recipe from base64 {e}");
        StatusCode::BAD_REQUEST
    })?;

    let recipe_pantry: Value = serde_json::from_slice(&de).map_err(|e| {
        eprintln!("ERR: error while deserialize from json {e}");
        StatusCode::BAD_REQUEST
    })?;
    Ok(Json(recipe_pantry))
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RecipePantry {
    recipe: HashMap<String, usize>,
    pantry: HashMap<String, usize>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CookieResult {
    cookies: usize,
    pantry: HashMap<String, usize>,
}

async fn secret_cookie(
    TypedHeader(cookie): TypedHeader<Cookie>,
) -> Result<Json<CookieResult>, StatusCode> {
    let recipe = cookie.get("recipe").ok_or(StatusCode::BAD_REQUEST)?;

    let de = general_purpose::STANDARD.decode(recipe).map_err(|e| {
        eprintln!("ERR: error while decoding recipe from base64 {e}");
        StatusCode::BAD_REQUEST
    })?;

    let mut recipe_pantry: RecipePantry = serde_json::from_slice(&de).map_err(|e| {
        eprintln!("ERR: error while deserialize from json {e}");
        StatusCode::BAD_REQUEST
    })?;

    let cookies_count = recipe_pantry
        .recipe
        .iter()
        .map(|(recipe_key, &pantry_needed)| {
            if pantry_needed == 0 {
                usize::MAX
            } else {
                recipe_pantry
                    .pantry
                    .get(recipe_key)
                    .map(|&pantry_available| pantry_available / pantry_needed)
                    .unwrap_or_default()
            }
        })
        .min()
        .unwrap_or_default();

    for (recipe_key, &pantry_needed) in &recipe_pantry.recipe {
        if let Some(p) = recipe_pantry.pantry.get_mut(recipe_key) {
            *p -= cookies_count * pantry_needed;
        }
    }

    let result = CookieResult {
        cookies: cookies_count,
        pantry: recipe_pantry.pantry,
    };
    Ok(Json(result))
}
