use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};
use cch23_challenge::handlers::day22::router;
use http_body_util::BodyExt;
use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

// TODO: remove the coupling between the tests and axum service
// TODO: like this this example https://github.com/LukeMathWalker/zero-to-production/blob/root-chapter-03-part1/tests/health_check.rs
#[tokio::test]
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
