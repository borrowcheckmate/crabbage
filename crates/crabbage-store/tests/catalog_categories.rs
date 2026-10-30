//! Recipe categories: listed by name, unique regardless of case, renamed, and deleted from every
//! recipe they were filed under.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::StoreError;
use crabbage_store::catalog::categories::{create, delete, list, rename};
use crabbage_store::recipes::{RecipeInput, create as create_recipe, get};
use crabbage_store::users::upsert_user;
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
        .map(|category| category.name)
        .collect()
}

#[tokio::test]
async fn categories_are_listed_by_name() {
    let (_dir, pool) = database("categories-list").await;
    create(&pool, "Dinner").await.unwrap();
    create(&pool, "breakfast").await.unwrap();
    create(&pool, "Afternoon").await.unwrap();
    assert_eq!(names(&pool).await, ["Afternoon", "breakfast", "Dinner"]);
}

#[tokio::test]
async fn category_names_are_unique_regardless_of_case() {
    let (_dir, pool) = database("categories-unique").await;
    let dinner = create(&pool, "Dinner").await.unwrap();
    create(&pool, "Breakfast").await.unwrap();
    assert!(matches!(
        create(&pool, "dinner").await.unwrap_err(),
        StoreError::Conflict(_)
    ));
    assert!(matches!(
        rename(&pool, dinner, "BREAKFAST").await.unwrap_err(),
        StoreError::Conflict(_)
    ));
    assert!(rename(&pool, dinner, "Supper").await.unwrap());
    assert!(!rename(&pool, 999, "Ghost").await.unwrap());
    assert_eq!(names(&pool).await, ["Breakfast", "Supper"]);
}

#[tokio::test]
async fn deleting_a_category_takes_it_off_every_recipe() {
    let (_dir, pool) = database("categories-delete").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    let dinner = create(&pool, "Dinner").await.unwrap();
    let soup = create_recipe(
        &pool,
        owner,
        &RecipeInput {
            title: "Soup".to_owned(),
            servings: 2,
            lines: vec![],
            steps: vec![],
            category_ids: vec![dinner],
        },
        0,
    )
    .await
    .unwrap();

    assert!(delete(&pool, dinner).await.unwrap());
    assert!(!delete(&pool, dinner).await.unwrap());

    assert_eq!(names(&pool).await, Vec::<String>::new());
    let recipe = get(&pool, soup).await.unwrap().expect("the recipe stays");
    assert_eq!(recipe.categories, Vec::new());
}
