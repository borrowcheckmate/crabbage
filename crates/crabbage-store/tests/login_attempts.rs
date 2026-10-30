//! Pending login storage: taking an attempt once, and expiry after ten minutes.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::login_attempts::{self, LoginAttempt};

const NOW: i64 = 10_000;

fn attempt() -> LoginAttempt {
    LoginAttempt {
        nonce: "nonce".to_owned(),
        pkce_verifier: "verifier".to_owned(),
    }
}

#[tokio::test]
async fn take_returns_the_attempt_once() {
    let (_dir, pool) = database("attempt-take").await;
    login_attempts::create(&pool, "state", &attempt(), NOW)
        .await
        .unwrap();
    assert_eq!(
        login_attempts::take(&pool, "state", NOW + 1).await.unwrap(),
        Some(attempt())
    );
    assert_eq!(
        login_attempts::take(&pool, "state", NOW + 1).await.unwrap(),
        None
    );
}

#[tokio::test]
async fn unknown_state_is_not_found() {
    let (_dir, pool) = database("attempt-unknown").await;
    login_attempts::create(&pool, "state", &attempt(), NOW)
        .await
        .unwrap();
    assert_eq!(
        login_attempts::take(&pool, "other", NOW).await.unwrap(),
        None
    );
}

#[tokio::test]
async fn expired_attempt_is_not_returned() {
    let (_dir, pool) = database("attempt-expired").await;
    login_attempts::create(&pool, "state", &attempt(), NOW)
        .await
        .unwrap();
    assert_eq!(
        login_attempts::take(&pool, "state", NOW + 600)
            .await
            .unwrap(),
        None
    );
}

#[tokio::test]
async fn attempt_is_returned_just_before_it_expires() {
    let (_dir, pool) = database("attempt-edge").await;
    login_attempts::create(&pool, "state", &attempt(), NOW)
        .await
        .unwrap();
    assert_eq!(
        login_attempts::take(&pool, "state", NOW + 599)
            .await
            .unwrap(),
        Some(attempt())
    );
}

#[tokio::test]
async fn create_drops_expired_attempts() {
    let (_dir, pool) = database("attempt-purge").await;
    login_attempts::create(&pool, "old", &attempt(), NOW)
        .await
        .unwrap();
    login_attempts::create(&pool, "new", &attempt(), NOW + 600)
        .await
        .unwrap();
    assert_eq!(
        login_attempts::take(&pool, "old", NOW + 600).await.unwrap(),
        None
    );
    assert_eq!(
        login_attempts::take(&pool, "new", NOW + 600).await.unwrap(),
        Some(attempt())
    );
}
