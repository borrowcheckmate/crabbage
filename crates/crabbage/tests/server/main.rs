//! Checks the HTTP routes: the health endpoint, the server-rendered shell and sign-in.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod catalog;
#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;
mod cook_mode;
#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod discovery;
#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod household;
#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod ingredient_pages;
#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod login;
#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod pages;
mod recipe_browse;
#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod recipe_images;
#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod recipe_import;
mod recipe_pages;
#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod recipe_photo_import;
mod recipe_ratings;
mod recipe_step_lines;
#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod recipes;
mod screenshots;
mod session;
#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod shopping_live;
#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod shopping_pages;

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
    assert!(body.contains("<html lang=\"en\""), "{body}");
    assert!(body.contains("<title>crabbage</title>"), "{body}");
    assert!(body.contains("Sign out"), "{body}");
}

#[tokio::test]
async fn home_links_the_base_stylesheet_for_a_signed_in_user() {
    let harness = Harness::start("server-stylesheet").await;
    let token = session_token(&harness.sign_in().await).expect("sign-in sets a session");
    let body = body_text(harness.get("/", Some(&token)).await).await;
    assert!(body.contains("href=\"/pkg/crabbage.css\""), "{body}");
}

#[tokio::test]
async fn home_renders_in_the_configured_language() {
    let harness = Harness::start_with_language("server-nl", Some("nl")).await;
    let token = session_token(&harness.sign_in().await).expect("sign-in sets a session");
    let response = harness.get("/", Some(&token)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("<html lang=\"nl\""), "{body}");
    assert!(body.contains("<title>crabbage</title>"), "{body}");
    assert!(body.contains("Uitloggen"), "{body}");
    assert!(!body.contains("Sign out"), "{body}");
}
