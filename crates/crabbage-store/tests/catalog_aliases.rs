//! Aliases: unique regardless of case, never equal to an ingredient name, and removable.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::StoreError;
use crabbage_store::catalog::ingredients::{add_alias, create, list, remove_alias};

#[tokio::test]
async fn aliases_are_listed_with_their_ingredient() {
    let (_dir, pool) = database("alias-add").await;
    let id = create(&pool, "Tomato").await.unwrap();
    assert!(add_alias(&pool, id, "Pomodoro").await.unwrap());
    assert!(add_alias(&pool, id, "Love apple").await.unwrap());
    let ingredients = list(&pool).await.unwrap();
    assert_eq!(ingredients[0].aliases, ["Love apple", "Pomodoro"]);
}

#[tokio::test]
async fn aliases_are_unique_regardless_of_case() {
    let (_dir, pool) = database("alias-case").await;
    let id = create(&pool, "Tomato").await.unwrap();
    let other = create(&pool, "Potato").await.unwrap();
    add_alias(&pool, id, "Tom").await.unwrap();
    assert!(matches!(
        add_alias(&pool, other, "tom").await.unwrap_err(),
        StoreError::Conflict(ref alias) if alias == "tom"
    ));
    assert!(matches!(
        add_alias(&pool, id, "TOM").await.unwrap_err(),
        StoreError::Conflict(_)
    ));
}

#[tokio::test]
async fn an_alias_cannot_equal_an_ingredient_name() {
    let (_dir, pool) = database("alias-name").await;
    let id = create(&pool, "Tomato").await.unwrap();
    create(&pool, "Potato").await.unwrap();
    assert!(matches!(
        add_alias(&pool, id, "potato").await.unwrap_err(),
        StoreError::Conflict(_)
    ));
    assert!(matches!(
        add_alias(&pool, id, "TOMATO").await.unwrap_err(),
        StoreError::Conflict(_)
    ));
}

#[tokio::test]
async fn alias_for_a_missing_ingredient_is_not_added() {
    let (_dir, pool) = database("alias-missing").await;
    assert!(!add_alias(&pool, 999, "Ghost").await.unwrap());
}

#[tokio::test]
async fn remove_alias_only_removes_it_from_its_ingredient() {
    let (_dir, pool) = database("alias-remove").await;
    let id = create(&pool, "Tomato").await.unwrap();
    let other = create(&pool, "Potato").await.unwrap();
    add_alias(&pool, id, "Pomodoro").await.unwrap();
    assert!(!remove_alias(&pool, other, "Pomodoro").await.unwrap());
    assert!(remove_alias(&pool, id, "POMODORO").await.unwrap());
    assert!(!remove_alias(&pool, id, "Pomodoro").await.unwrap());
}
