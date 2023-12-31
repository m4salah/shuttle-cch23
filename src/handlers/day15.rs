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
    (StatusCode::BAD_REQUEST, Json(json!({"result": "naughty"})))
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
        if c.is_ascii_digit() {
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
        if ('\u{2980}'..='\u{2BFF}').contains(&c) {
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PasswordGameResult<'a> {
    result: &'a str,
    reason: &'a str,
}

async fn game_validator(Json(password_input): Json<PasswordInput>) -> impl IntoResponse {
    tracing::info!("game validator called with {password_input:?}");

    // Rule 1: must be at least 8 characters long
    if password_input.input.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(PasswordGameResult {
                result: "naughty",
                reason: "8 chars",
            }),
        );
    }

    // Rule 2: must contain uppercase letters, lowercase letters, and digits
    if !contains_upper_lower_digit(&password_input.input) {
        return (
            StatusCode::BAD_REQUEST,
            Json(PasswordGameResult {
                result: "naughty",
                reason: "more types of chars",
            }),
        );
    }

    // Rule 3: must contain at least 5 digits
    if !contains_at_least_five_digits(&password_input.input) {
        return (
            StatusCode::BAD_REQUEST,
            Json(PasswordGameResult {
                result: "naughty",
                reason: "55555",
            }),
        );
    }

    // Rule 4: all integers (sequences of consecutive digits) in the string must add up to 2023
    if !integers_sum_to_2023(&password_input.input) {
        return (
            StatusCode::BAD_REQUEST,
            Json(PasswordGameResult {
                result: "naughty",
                reason: "math is hard",
            }),
        );
    }

    // Rule 5: must contain the letters j, o, and y in that order and in no other order
    if !contains_j_o_y_in_order(&password_input.input) {
        return (
            StatusCode::NOT_ACCEPTABLE,
            Json(PasswordGameResult {
                result: "naughty",
                reason: "not joyful enough",
            }),
        );
    }

    // Rule 6: must contain a letter that repeats with exactly one other letter between them (like xyx)
    if !contains_sandwich(&password_input.input) {
        return (
            StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS,
            Json(PasswordGameResult {
                result: "naughty",
                reason: "illegal: no sandwich",
            }),
        );
    }

    // Rule 7: must contain at least one unicode character in the range [U+2980, U+2BFF]
    if !contains_unicode_in_range(&password_input.input) {
        return (
            StatusCode::RANGE_NOT_SATISFIABLE,
            Json(PasswordGameResult {
                result: "naughty",
                reason: "outranged",
            }),
        );
    }

    // Rule 8: must contain at least one emoji
    if !contains_emoji(&password_input.input) {
        return (
            StatusCode::UPGRADE_REQUIRED,
            Json(PasswordGameResult {
                result: "naughty",
                reason: "😳",
            }),
        );
    }

    // Rule 9: the hexadecimal representation of the sha256 hash of the string must end with an a
    if !hash_ends_with_a(&password_input.input) {
        return (
            StatusCode::IM_A_TEAPOT,
            Json(PasswordGameResult {
                result: "naughty",
                reason: "not a coffee brewer",
            }),
        );
    }

    (
        StatusCode::OK,
        Json(PasswordGameResult {
            result: "nice",
            reason: "that's a nice password",
        }),
    )
}
