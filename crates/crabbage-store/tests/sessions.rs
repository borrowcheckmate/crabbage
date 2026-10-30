//! Session storage: creating, finding, expiring and deleting sessions.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::{sessions, users};

const NOW: i64 = 1_000;

#[tokio::test]
async fn created_session_is_found_until_it_expires() {
    let (_dir, pool) = database("session-find").await;
    let user = users::upsert_user(&pool, "subject", "Name").await.unwrap();
    sessions::create(&pool, "token", user, NOW + 60)
        .await
        .unwrap();
    assert_eq!(
        sessions::find_user(&pool, "token", NOW).await.unwrap(),
        Some(user)
    );
    assert_eq!(
        sessions::find_user(&pool, "token", NOW + 59).await.unwrap(),
        Some(user)
    );
    assert_eq!(
        sessions::find_user(&pool, "token", NOW + 60).await.unwrap(),
        None
    );
}

#[tokio::test]
async fn expired_session_is_not_found() {
    let (_dir, pool) = database("session-expired").await;
    let user = users::upsert_user(&pool, "subject", "Name").await.unwrap();
    sessions::create(&pool, "token", user, NOW - 1)
        .await
        .unwrap();
    assert_eq!(
        sessions::find_user(&pool, "token", NOW).await.unwrap(),
        None
    );
}

#[tokio::test]
async fn unknown_token_is_not_found() {
    let (_dir, pool) = database("session-unknown").await;
    let user = users::upsert_user(&pool, "subject", "Name").await.unwrap();
    sessions::create(&pool, "token", user, NOW + 60)
        .await
        .unwrap();
    assert_eq!(
        sessions::find_user(&pool, "other", NOW).await.unwrap(),
        None
    );
}

#[tokio::test]
async fn delete_removes_the_session() {
    let (_dir, pool) = database("session-delete").await;
    let user = users::upsert_user(&pool, "subject", "Name").await.unwrap();
    sessions::create(&pool, "token", user, NOW + 60)
        .await
        .unwrap();
    sessions::delete(&pool, "token").await.unwrap();
    assert_eq!(
        sessions::find_user(&pool, "token", NOW).await.unwrap(),
        None
    );
    sessions::delete(&pool, "token").await.unwrap();
}

#[tokio::test]
async fn session_for_unknown_user_is_rejected() {
    let (_dir, pool) = database("session-no-user").await;
    assert!(
        sessions::create(&pool, "token", 404, NOW + 60)
            .await
            .is_err()
    );
}
