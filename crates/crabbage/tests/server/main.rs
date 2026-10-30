//! Checks the HTTP routes: the health endpoint, the server-rendered shell and sign-in.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;
mod login;
mod session;

use axum::http::StatusCode;
use common::{Harness, body_text, session_token};

#[tokio::test]
async fn healthz_is_ok_without_a_session() {
    let harness = Harness::start("server-healthz").await;
    let response = harness.get("/healthz", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_text(response).await, "ok");
}

#[tokio::test]
async fn home_renders_the_shell_with_the_title_for_a_signed_in_user() {
    let harness = Harness::start("server-home").await;
    let token = session_token(&harness.sign_in().await).expect("sign-in sets a session");
    assert_eq!(harness.session_count().await, 1);
    let response = harness.get("/", Some(&token)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("<title>crabbage</title>"), "{body}");
}
