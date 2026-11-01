//! Households: created on first sight, memberships replaced on each login, the active household
//! kept, defaulted and checked against the memberships.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::households::{
    active_household, households_of, list, rename, set_active_household, sync_membership,
};
use crabbage_store::users::upsert_user;
use sqlx::sqlite::SqlitePool;

fn groups(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

#[expect(
    clippy::unwrap_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn household_id(pool: &SqlitePool, user: i64, group: &str) -> i64 {
    households_of(pool, user)
        .await
        .unwrap()
        .into_iter()
        .find(|household| household.group_name == group)
        .unwrap()
        .id
}

#[tokio::test]
async fn a_household_is_created_on_first_sight_and_reused_by_another_user() {
    let (_dir, pool) = database("households-shared").await;
    let first = upsert_user(&pool, "one", "One").await.unwrap();
    let second = upsert_user(&pool, "two", "Two").await.unwrap();
    sync_membership(&pool, first, false, &groups(&["crabbage-household-smith"]))
        .await
        .unwrap();
    sync_membership(&pool, second, false, &groups(&["crabbage-household-smith"]))
        .await
        .unwrap();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM households")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
    assert_eq!(
        household_id(&pool, first, "crabbage-household-smith").await,
        household_id(&pool, second, "crabbage-household-smith").await
    );
}

#[tokio::test]
async fn sync_replaces_the_memberships_and_sets_the_admin_flag() {
    let (_dir, pool) = database("households-replace").await;
    let user = upsert_user(&pool, "subject", "Name").await.unwrap();
    sync_membership(&pool, user, false, &groups(&["a", "b"]))
        .await
        .unwrap();
    sync_membership(&pool, user, true, &groups(&["b", "c"]))
        .await
        .unwrap();
    let names: Vec<String> = households_of(&pool, user)
        .await
        .unwrap()
        .into_iter()
        .map(|household| household.group_name)
        .collect();
    assert_eq!(names, ["b", "c"]);
    let is_admin: bool = sqlx::query_scalar("SELECT is_admin FROM users WHERE id = ?")
        .bind(user)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(is_admin);
}

#[tokio::test]
async fn active_household_defaults_to_the_first_by_group_name() {
    let (_dir, pool) = database("households-default").await;
    let user = upsert_user(&pool, "subject", "Name").await.unwrap();
    sync_membership(&pool, user, false, &groups(&["b", "a"]))
        .await
        .unwrap();
    assert_eq!(
        active_household(&pool, user).await.unwrap(),
        Some(household_id(&pool, user, "a").await)
    );
}

#[tokio::test]
async fn removing_the_active_household_moves_to_the_first_remaining() {
    let (_dir, pool) = database("households-removed").await;
    let user = upsert_user(&pool, "subject", "Name").await.unwrap();
    sync_membership(&pool, user, false, &groups(&["a", "b"]))
        .await
        .unwrap();
    let b = household_id(&pool, user, "b").await;
    assert!(set_active_household(&pool, user, b).await.unwrap());
    sync_membership(&pool, user, false, &groups(&["a"]))
        .await
        .unwrap();
    assert_eq!(
        active_household(&pool, user).await.unwrap(),
        Some(household_id(&pool, user, "a").await)
    );
}

#[tokio::test]
async fn set_active_rejects_a_household_the_user_is_not_in() {
    let (_dir, pool) = database("households-reject").await;
    let user = upsert_user(&pool, "one", "One").await.unwrap();
    let other = upsert_user(&pool, "two", "Two").await.unwrap();
    sync_membership(&pool, user, false, &groups(&["a"]))
        .await
        .unwrap();
    sync_membership(&pool, other, false, &groups(&["b"]))
        .await
        .unwrap();
    let a = household_id(&pool, user, "a").await;
    let b = household_id(&pool, other, "b").await;
    assert!(!set_active_household(&pool, user, b).await.unwrap());
    assert_eq!(active_household(&pool, user).await.unwrap(), Some(a));
}

#[tokio::test]
async fn the_chosen_household_survives_a_sync_that_keeps_it() {
    let (_dir, pool) = database("households-persist").await;
    let user = upsert_user(&pool, "subject", "Name").await.unwrap();
    sync_membership(&pool, user, false, &groups(&["a", "b"]))
        .await
        .unwrap();
    let b = household_id(&pool, user, "b").await;
    assert!(set_active_household(&pool, user, b).await.unwrap());
    sync_membership(&pool, user, false, &groups(&["a", "b"]))
        .await
        .unwrap();
    assert_eq!(active_household(&pool, user).await.unwrap(), Some(b));
}

#[tokio::test]
async fn households_can_be_listed_and_renamed_or_cleared() {
    let (_dir, pool) = database("households-rename").await;
    let user = upsert_user(&pool, "subject", "Name").await.unwrap();
    sync_membership(&pool, user, false, &groups(&["b", "a"]))
        .await
        .unwrap();
    let rows = list(&pool).await.unwrap();
    assert_eq!(
        rows.iter()
            .map(|row| row.group_name.as_str())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
    let a = rows[0].id;
    assert!(rename(&pool, a, Some("  Maple Street  ")).await.unwrap());
    assert_eq!(
        list(&pool).await.unwrap()[0].name.as_deref(),
        Some("Maple Street")
    );
    assert!(rename(&pool, a, Some("  ")).await.unwrap());
    assert_eq!(list(&pool).await.unwrap()[0].name, None);
    assert!(!rename(&pool, -1, Some("Missing")).await.unwrap());
}
