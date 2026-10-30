//! User upserts: one row per subject, the name follows the latest login.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::users::{find, upsert_user};

#[tokio::test]
async fn same_subject_keeps_its_id_and_takes_the_new_name() {
    let (_dir, pool) = database("user-upsert").await;
    let first = upsert_user(&pool, "subject", "Old name").await.unwrap();
    let second = upsert_user(&pool, "subject", "New name").await.unwrap();
    assert_eq!(first, second);
    let name: String = sqlx::query_scalar("SELECT name FROM users WHERE id = ?")
        .bind(first)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(name, "New name");
}

#[tokio::test]
async fn different_subjects_get_different_ids() {
    let (_dir, pool) = database("user-distinct").await;
    let first = upsert_user(&pool, "one", "Name").await.unwrap();
    let second = upsert_user(&pool, "two", "Name").await.unwrap();
    assert_ne!(first, second);
}

#[tokio::test]
async fn find_returns_the_name_and_admin_flag_or_none() {
    let (_dir, pool) = database("user-find").await;
    let id = upsert_user(&pool, "subject", "Name").await.unwrap();
    sqlx::query("UPDATE users SET is_admin = 1 WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    let user = find(&pool, id).await.unwrap().unwrap();
    assert_eq!(user.name, "Name");
    assert!(user.is_admin);
    assert!(find(&pool, id + 1).await.unwrap().is_none());
}
