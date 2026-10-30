//! Store sections: created at the end, unique regardless of case, reordered to 0..n, and deleting
//! one leaves its ingredients without a section.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::StoreError;
use crabbage_store::catalog::ingredients::{
    create as create_ingredient, list as list_ingredients, set_section,
};
use crabbage_store::catalog::sections::{create, delete, list, move_section, rename};
use sqlx::sqlite::SqlitePool;

#[expect(
    clippy::unwrap_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn names(pool: &SqlitePool) -> Vec<String> {
    list(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|section| section.name)
        .collect()
}

#[tokio::test]
async fn sections_are_listed_in_the_order_they_were_created() {
    let (_dir, pool) = database("section-create").await;
    create(&pool, "Dairy").await.unwrap();
    create(&pool, "Bakery").await.unwrap();
    assert_eq!(names(&pool).await, ["Dairy", "Bakery"]);
    let positions: Vec<i64> = list(&pool)
        .await
        .unwrap()
        .into_iter()
        .map(|section| section.position)
        .collect();
    assert_eq!(positions, [0, 1]);
}

#[tokio::test]
async fn section_names_are_unique_regardless_of_case() {
    let (_dir, pool) = database("section-unique").await;
    let dairy = create(&pool, "Dairy").await.unwrap();
    create(&pool, "Bakery").await.unwrap();
    assert!(matches!(
        create(&pool, "dairy").await.unwrap_err(),
        StoreError::Conflict(_)
    ));
    assert!(matches!(
        rename(&pool, dairy, "BAKERY").await.unwrap_err(),
        StoreError::Conflict(_)
    ));
    assert!(rename(&pool, dairy, "Cheese").await.unwrap());
    assert!(!rename(&pool, 999, "Ghost").await.unwrap());
    assert_eq!(names(&pool).await, ["Cheese", "Bakery"]);
}

#[tokio::test]
async fn sections_move_and_renumber_from_zero() {
    let (_dir, pool) = database("section-move").await;
    create(&pool, "Dairy").await.unwrap();
    create(&pool, "Bakery").await.unwrap();
    let frozen = create(&pool, "Frozen").await.unwrap();

    assert!(move_section(&pool, frozen, 0).await.unwrap());
    assert_eq!(names(&pool).await, ["Frozen", "Dairy", "Bakery"]);

    assert!(move_section(&pool, frozen, 1).await.unwrap());
    assert_eq!(names(&pool).await, ["Dairy", "Frozen", "Bakery"]);

    assert!(move_section(&pool, frozen, 99).await.unwrap());
    assert_eq!(names(&pool).await, ["Dairy", "Bakery", "Frozen"]);

    assert!(move_section(&pool, frozen, -5).await.unwrap());
    assert_eq!(names(&pool).await, ["Frozen", "Dairy", "Bakery"]);

    let positions: Vec<i64> = list(&pool)
        .await
        .unwrap()
        .into_iter()
        .map(|section| section.position)
        .collect();
    assert_eq!(positions, [0, 1, 2]);
    assert!(!move_section(&pool, 999, 0).await.unwrap());
}

#[tokio::test]
async fn deleting_a_section_leaves_its_ingredients_without_one() {
    let (_dir, pool) = database("section-delete").await;
    let dairy = create(&pool, "Dairy").await.unwrap();
    let milk = create_ingredient(&pool, "Milk").await.unwrap();
    set_section(&pool, milk, Some(dairy)).await.unwrap();

    assert!(delete(&pool, dairy).await.unwrap());
    assert!(!delete(&pool, dairy).await.unwrap());

    assert_eq!(names(&pool).await.len(), 0);
    assert_eq!(
        list_ingredients(&pool).await.unwrap()[0].store_section_id,
        None
    );
}
