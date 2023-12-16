use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use crypto::{digest::Digest, sha2::Sha256};
use fancy_regex::Regex;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub fn router() -> Router {
    Router::new()
        .route("/15/health", get(|| async { StatusCode::OK }))
        .route("/15/nice", post(nice_validator))
        .route("/15/game", post(game_validator))
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PasswordInput {
    input: String,
}
async fn nice_validator(Json(password_input): Json<PasswordInput>) -> impl IntoResponse {
    tracing::info!("nice validator called with {password_input:?}");
    let three_vowls = Regex::new(r"^(.*[aeiouy]){3,}.*$").unwrap();

    let two_letter_appears_twice = Regex::new(r"([a-zA-Z])\1").unwrap();

    let not_contain = Regex::new(r"^(?:(?!ab|cd|pq|xy).)*$").unwrap();
    if let (Ok(t), Ok(two), Ok(no)) = (
        three_vowls.is_match(&password_input.input),
        two_letter_appears_twice.is_match(&password_input.input),
        not_contain.is_match(&password_input.input),
    ) {
        if t && two && no {
            return (StatusCode::OK, Json(json!({"result": "nice"})));
        }
    }
    return (StatusCode::BAD_REQUEST, Json(json!({"result": "naughty"})));
}

fn contains_upper_lower_digit(s: &str) -> bool {
    let re = Regex::new(r"^(?=.*[A-Z])(?=.*[a-z])(?=.*\d).+$").unwrap();
    if let Ok(m) = re.is_match(s) {
        return m;
    }
    false
}

fn contains_at_least_five_digits(s: &str) -> bool {
    let re = Regex::new(r"^(.*\d.*){5,}$").unwrap();
    if let Ok(m) = re.is_match(s) {
        return m;
    }
    false
}

fn integers_sum_to_2023(s: &str) -> bool {
    let mut current_num = String::new();
    let mut total_sum = 0;

    for c in s.chars() {
        if let Some(_) = c.to_digit(10) {
            current_num.push(c);
        } else {
            total_sum += current_num.parse::<u32>().unwrap_or_default();
            current_num.clear();
        }
    }

    total_sum == 2023
}

fn contains_j_o_y_in_order(s: &str) -> bool {
    let re = Regex::new(r"^.*j.+o.+y.*$").unwrap();
    if let Ok(m) = re.is_match(s) {
        return m;
    }
    false
}

fn contains_sandwich(s: &str) -> bool {
    for (a, b, c) in s.chars().tuple_windows() {
        if a.is_ascii_alphabetic() && b.is_ascii_alphabetic() && c.is_ascii_alphabetic() && a == c {
            return true;
        }
    }
    false
}

fn contains_unicode_in_range(s: &str) -> bool {
    for c in s.chars() {
        if c >= '\u{2980}' && c <= '\u{2BFF}' {
            return true;
        }
    }
    false
}

fn contains_emoji(s: &str) -> bool {
    let re = Regex::new(r"\p{Extended_Pictographic}").unwrap();
    if let Ok(m) = re.is_match(s) {
        return m;
    }
    false
}

fn hash_ends_with_a(s: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.input_str(s);
    let hash = hasher.result_str();

    hash.ends_with('a')
}

async fn game_validator(Json(password_input): Json<PasswordInput>) -> impl IntoResponse {
    tracing::info!("game validator called with {password_input:?}");

    // Rule 1: must be at least 8 characters long
    if password_input.input.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"result": "naughty", "reason": "8 chars"})),
        );
    }

    // Rule 2: must contain uppercase letters, lowercase letters, and digits
    if !contains_upper_lower_digit(&password_input.input) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"result": "naughty", "reason": "more types of chars"})),
        );
    }

    // Rule 3: must contain at least 5 digits
    if !contains_at_least_five_digits(&password_input.input) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"result": "naughty", "reason": "55555"})),
        );
    }

    // Rule 4: all integers (sequences of consecutive digits) in the string must add up to 2023
    if !integers_sum_to_2023(&password_input.input) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"result": "naughty", "reason": "math is hard"})),
        );
    }

    // Rule 5: must contain the letters j, o, and y in that order and in no other order
    if !contains_j_o_y_in_order(&password_input.input) {
        return (
            StatusCode::NOT_ACCEPTABLE,
            Json(json!({"result": "naughty", "reason": "not joyful enough"})),
        );
    }

    // Rule 6: must contain a letter that repeats with exactly one other letter between them (like xyx)
    if !contains_sandwich(&password_input.input) {
        return (
            StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS,
            Json(json!({"result": "naughty", "reason": "illegal: no sandwich"})),
        );
    }

    // Rule 7: must contain at least one unicode character in the range [U+2980, U+2BFF]
    if !contains_unicode_in_range(&password_input.input) {
        return (
            StatusCode::RANGE_NOT_SATISFIABLE,
            Json(json!({"result": "naughty", "reason": "outranged"})),
        );
    }

    // Rule 8: must contain at least one emoji
    if !contains_emoji(&password_input.input) {
        return (
            StatusCode::UPGRADE_REQUIRED,
            Json(json!({"result": "naughty", "reason": "😳"})),
        );
    }

    // Rule 9: the hexadecimal representation of the sha256 hash of the string must end with an a
    if !hash_ends_with_a(&password_input.input) {
        return (
            StatusCode::IM_A_TEAPOT,
            Json(json!({"result": "naughty", "reason": "not a coffee brewer"})),
        );
    }

    return (
        StatusCode::OK,
        Json(json!({"result": "nice", "reason": "that's a nice password"})),
    );
}

#[cfg(test)]
mod tests {

    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;
    use reqwest::header::CONTENT_TYPE;
    use serde_json::{json, Value};

    #[tokio::test]
    async fn day15_health() {
        let app = router();
        let client = TestClient::new(app);
        let res = client.get("/15/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day15_nice() {
        let app = router();
        let client = TestClient::new(app);
        let res = client
            .post("/15/nice")
            .body(
                json!({
                "input": "hello there"
                })
                .to_string(),
            )
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);

        let expected = json!({"result":"nice"});
        assert_eq!(res.json::<Value>().await, expected);
    }

    #[tokio::test]
    async fn day15_naughty() {
        let app = router();
        let client = TestClient::new(app);
        let res = client
            .post("/15/nice")
            .body(
                json!({
                "input": "abcd"
                })
                .to_string(),
            )
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        let expected = json!({"result":"naughty"});
        assert_eq!(res.json::<Value>().await, expected);
    }

    #[tokio::test]
    async fn day15_invalid_json() {
        let app = router();
        let client = TestClient::new(app);
        let res = client
            .post("/15/nice")
            .body("{Grinch? GRINCH!}")
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn day15_game_nice_more_chars() {
        let app = router();
        let client = TestClient::new(app);
        let res = client
            .post("/15/game")
            .body(json!({"input": "password"}).to_string())
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            res.json::<Value>().await,
            json!({"result":"naughty","reason":"more types of chars"})
        );
    }

    #[tokio::test]
    async fn day15_game_nice_math_hard() {
        let app = router();
        let client = TestClient::new(app);
        let res = client
            .post("/15/game")
            .body(json!({"input": "Password12345"}).to_string())
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            res.json::<Value>().await,
            json!({"result":"naughty","reason":"math is hard"})
        );
    }

    #[tokio::test]
    async fn day15_game_nice_no_sandwich() {
        let app = router();
        let client = TestClient::new(app);
        let res = client
            .post("/15/game")
            .body(json!({"input": "23jPassword2000y"}).to_string())
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS);
        assert_eq!(
            res.json::<Value>().await,
            json!({"result":"naughty","reason":"illegal: no sandwich"})
        );
    }

    #[tokio::test]
    async fn test_contains_emoji() {
        assert_eq!(contains_emoji("hello"), false);
        assert_eq!(contains_emoji("hello 😳"), true);
        assert_eq!(contains_emoji("2000.23.A j  ;)  o  ;)  y ⦄AzA"), false);
    }

    #[tokio::test]
    async fn test_joy() {
        assert_eq!(contains_j_o_y_in_order("2000.23.A joy joy"), false);
        assert_eq!(contains_j_o_y_in_order("2020.3.A j  ;)  o  ;)  y"), true);
    }

    #[tokio::test]
    async fn test_contains_five_digits() {
        assert_eq!(contains_at_least_five_digits("hello"), false);
        assert_eq!(contains_at_least_five_digits("123hello"), false);
        assert_eq!(contains_at_least_five_digits("12345hello"), true);
        assert_eq!(contains_at_least_five_digits("1skdj3skdjf34jskdjf4"), true);
    }
}
