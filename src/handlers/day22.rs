use std::collections::HashSet;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use itertools::Itertools;
use tracing::info;

pub fn router() -> Router {
    Router::new()
        .route("/22/health", get(|| async { StatusCode::OK }))
        .route("/22/integers", post(integers))
        .route("/22/rocket", post(rocket))
}

async fn integers(content: String) -> impl IntoResponse {
    let mut result: HashSet<usize> = HashSet::new();
    for line in content.lines() {
        if let Ok(num) = line.trim().parse::<usize>() {
            if result.take(&num).is_none() {
                result.insert(num);
            }
        }
    }
    let rep = result.into_iter().next().unwrap_or_default();
    "🎁".repeat(rep)
}

async fn rocket(content: String) -> impl IntoResponse {
    let mut content_lines = content.lines();
    let star_nums: u32 = content_lines.next().unwrap().parse().unwrap();
    let mut stars = Vec::new();
    let mut portals = Vec::new();

    for _ in 0..star_nums {
        let s = content_lines
            .next()
            .unwrap()
            .splitn(3, ' ')
            .map(|s| s.parse::<i32>().unwrap())
            .collect_tuple::<(i32, i32, i32)>()
            .unwrap();
        stars.push(s);
    }
    let portal_nums: u32 = content_lines.next().unwrap().parse().unwrap();
    for _ in 0..portal_nums {
        let s = content_lines
            .next()
            .unwrap()
            .splitn(2, ' ')
            .map(|s| s.parse::<i32>().unwrap())
            .collect_tuple::<(i32, i32)>()
            .unwrap();
        portals.push(s);
    }

    info!("{star_nums}: {:?}", stars);
    info!("{portal_nums}: {:?}", portals);
    info!("{}", content);
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{header, Method, Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt; // for `call`, `oneshot`, and `ready`
    use tracing_test::traced_test;

    #[tokio::test]
    #[traced_test]
    async fn health_check() {
        // Arrange
        let app = router();

        // Act
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/22/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[traced_test]
    async fn day22_integers() {
        // Arrange
        let app = router();

        // Act
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/22/integers")
                    .method(Method::POST)
                    .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(
                        r#"888
77
888
22
77"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = String::from_utf8_lossy(&body[..]);
        let expected = "🎁".repeat(22);
        assert_eq!(body, expected);
    }

    #[tokio::test]
    #[traced_test]
    async fn day22_rocket() {
        // Arrange
        let app = router();

        // Act
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/22/rocket")
                    .method(Method::POST)
                    .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(
                        r#"5
0 1 0
-2 2 3
3 -3 -5
1 1 5
4 3 5
4
0 1
2 4
3 4
1 2
"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let expected = "3 26.123";
        let body = String::from_utf8_lossy(&body[..]);
        assert_eq!(body, expected);
    }
}
