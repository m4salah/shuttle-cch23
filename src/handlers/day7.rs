use axum::{
    http::{header::COOKIE, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json,
};
use base64::{engine::general_purpose, Engine};
use cookie::Cookie;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/7/decode", get(santa_cookie))
        .route("/7/bake", get(secret_cookie))
}

async fn santa_cookie(headers: HeaderMap) -> impl IntoResponse {
    let cookies_string = headers.get(COOKIE).unwrap().to_str().unwrap();
    let mut result = None;

    for cookie in Cookie::split_parse(cookies_string) {
        if let Ok(c) = cookie {
            match c.name() {
                "recipe" => result = Some(c.value().to_owned()),
                _ => {}
            }
        }
    }
    if let Some(res) = result {
        let de = general_purpose::STANDARD.decode(res).unwrap();
        return Json(serde_json::from_slice::<Value>(&de).unwrap()).into_response();
    }
    (StatusCode::BAD_REQUEST, "bad request").into_response()
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipePantry {
    pub recipe: Value,
    pub pantry: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CookieResult {
    pub cookies: i64,
    pub pantry: Value,
}

async fn secret_cookie(headers: HeaderMap) -> impl IntoResponse {
    let cookies_string = headers.get(COOKIE).unwrap().to_str().unwrap();
    let mut result = None;

    for cookie in Cookie::split_parse(cookies_string) {
        if let Ok(c) = cookie {
            match c.name() {
                "recipe" => result = Some(c.value().to_owned()),
                _ => {}
            }
        }
    }
    if let Some(res) = result {
        let de = general_purpose::STANDARD.decode(res).unwrap();
        let req: RecipePantry = serde_json::from_slice(&de).unwrap();

        let cookies_count = match (req.recipe.clone(), req.pantry.clone()) {
            (Value::Object(recipe_map), Value::Object(pantry_map)) => recipe_map
                .into_iter()
                .map(|(recipe_key, recipe_value)| {
                    if let (Some(Value::Number(pantry_needed)), Value::Number(recipe_value)) =
                        (pantry_map.get(&recipe_key), recipe_value)
                    {
                        pantry_needed.as_i64().unwrap() / recipe_value.as_i64().unwrap()
                    } else {
                        0
                    }
                })
                .min()
                .unwrap(),
            (_, _) => 0,
        };

        let rest_pantry = match (req.recipe, req.pantry.clone()) {
            (Value::Object(recipe_map), Value::Object(pantry_map)) => {
                let m: Map<String, Value> = recipe_map
                    .into_iter()
                    .filter_map(|(recipe_key, recipe_value)| {
                        if let (
                            Some(Value::Number(pantry_available)),
                            Value::Number(recipe_value),
                        ) = (pantry_map.get(&recipe_key), recipe_value)
                        {
                            //     flour: req.pantry.flour - (req.recipe.flour * cookies_count),
                            let pantry_available = pantry_available.as_i64().unwrap();
                            Some((
                                recipe_key,
                                Value::Number(Number::from(
                                    pantry_available
                                        - (recipe_value.as_i64().unwrap() * cookies_count),
                                )),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect();
                println!("{m:?}");
                Value::Object(m)
            }
            (_, _) => Value::Null,
        };
        let rest_pantry_is_empty = rest_pantry.as_object().unwrap().is_empty();
        let result = CookieResult {
            cookies: cookies_count,
            pantry: if rest_pantry_is_empty {
                req.pantry
            } else {
                rest_pantry
            },
        };
        // println!("{req:?}");
        return Json(result).into_response();
    }
    (StatusCode::BAD_REQUEST, "bad request").into_response()
}
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;
    use serde_json::json;

    #[tokio::test]
    async fn santa_cookie() {
        let app = router();

        let client = TestClient::new(app);
        let res = client
            .get("/7/decode")
            .header(
                "Cookie",
                "recipe=eyJmbG91ciI6MTAwLCJjaG9jb2xhdGUgY2hpcHMiOjIwfQ==",
            )
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = json!({
            "flour":100,
            "chocolate chips":20
        });
        assert_eq!(res.json::<Value>().await, expected);
    }

    #[tokio::test]
    async fn santa_cookie_bake() {
        let app = router();

        let client = TestClient::new(app);
        let res = client
            .get("/7/bake")
            .header(
                "Cookie",
                "recipe=eyJyZWNpcGUiOnsiZmxvdXIiOjk1LCJzdWdhciI6NTAsImJ1dHRlciI6MzAsImJha2luZyBwb3dkZXIiOjEwLCJjaG9jb2xhdGUgY2hpcHMiOjUwfSwicGFudHJ5Ijp7ImZsb3VyI\
                jozODUsInN1Z2FyIjo1MDcsImJ1dHRlciI6MjEyMiwiYmFraW5nIHBvd2RlciI6ODY1LCJjaG9jb2xhdGUgY2hpcHMiOjQ1N319",
            )
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = json!({
          "cookies": 4,
          "pantry": {
            "flour": 5,
            "sugar": 307,
            "butter": 2002,
            "baking powder": 825,
            "chocolate chips": 257
        }});
        assert_eq!(res.json::<Value>().await, expected);
    }

    #[tokio::test]
    async fn santa_cookie_bake_base64() {
        let app = router();

        let client = TestClient::new(app);
        let res = client
            .get("/7/bake")
            .header(
                "Cookie",
                "recipe=eyJyZWNpcGUiOnsic2xpbWUiOjl9LCJwYW50cnkiOnsiY29iYmxlc3RvbmUiOjY0LCJzdGljayI6IDR9fQ==",
            )
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = json!({
        "cookies": 0,
        "pantry": {
          "cobblestone": 64,
          "stick": 4
        }});
        assert_eq!(res.json::<Value>().await, expected);
    }
}
