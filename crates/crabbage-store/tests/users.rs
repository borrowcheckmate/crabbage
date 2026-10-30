//! User upserts: one row per subject, the name follows the latest login.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::users::upsert_user;

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
