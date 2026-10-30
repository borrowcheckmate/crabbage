//! Ingredients: names are unique regardless of case, deletes are refused while recipes use them,
//! and the review queue lists and approves pending ones.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::StoreError;
use crabbage_store::catalog::ingredients::{
    add_alias, approve, create, delete, list, list_pending, rename, set_section,
};
use sqlx::sqlite::SqlitePool;

#[tokio::test]
async fn created_ingredients_are_listed_by_name_and_approved() {
    let (_dir, pool) = database("ingredient-create").await;
    create(&pool, "Onion").await.unwrap();
    create(&pool, "Carrot").await.unwrap();
    let names: Vec<String> = list(&pool)
        .await
        .unwrap()
        .into_iter()
        .map(|ingredient| ingredient.name)
        .collect();
    assert_eq!(names, ["Carrot", "Onion"]);
    assert_eq!(list_pending(&pool).await.unwrap().len(), 0);
}

#[tokio::test]
async fn names_are_unique_regardless_of_case() {
    let (_dir, pool) = database("ingredient-name-case").await;
    create(&pool, "Tomato").await.unwrap();
    let error = create(&pool, "tomato").await.unwrap_err();
    assert!(matches!(error, StoreError::Conflict(ref name) if name == "tomato"));
}

#[tokio::test]
async fn a_name_cannot_be_an_existing_alias() {
    let (_dir, pool) = database("ingredient-name-alias").await;
    let id = create(&pool, "Tomato").await.unwrap();
    add_alias(&pool, id, "Pomodoro").await.unwrap();
    let error = create(&pool, "pomodoro").await.unwrap_err();
    assert!(matches!(error, StoreError::Conflict(_)));
}

#[tokio::test]
async fn rename_keeps_the_id_and_refuses_taken_names() {
    let (_dir, pool) = database("ingredient-rename").await;
    let id = create(&pool, "Tomato").await.unwrap();
    create(&pool, "Onion").await.unwrap();
    assert!(rename(&pool, id, "Cherry tomato").await.unwrap());
    assert!(matches!(
        rename(&pool, id, "ONION").await.unwrap_err(),
        StoreError::Conflict(_)
    ));
    assert!(!rename(&pool, 999, "Ghost").await.unwrap());
    assert_eq!(list(&pool).await.unwrap()[0].name, "Cherry tomato");
}

#[tokio::test]
async fn rename_refuses_an_alias_of_the_same_ingredient() {
    let (_dir, pool) = database("ingredient-rename-alias").await;
    let id = create(&pool, "Tomato").await.unwrap();
    add_alias(&pool, id, "Tom").await.unwrap();
    assert!(matches!(
        rename(&pool, id, "tom").await.unwrap_err(),
        StoreError::Conflict(_)
    ));
}

#[tokio::test]
async fn delete_removes_the_ingredient_and_its_aliases() {
    let (_dir, pool) = database("ingredient-delete").await;
    let id = create(&pool, "Tomato").await.unwrap();
    add_alias(&pool, id, "Pomodoro").await.unwrap();
    assert!(delete(&pool, id).await.unwrap());
    assert!(!delete(&pool, id).await.unwrap());
    let aliases: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ingredient_aliases")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(aliases, 0);
}

#[tokio::test]
async fn delete_is_refused_while_recipe_lines_use_the_ingredient() {
    let (_dir, pool) = database("ingredient-in-use").await;
    let id = create(&pool, "Tomato").await.unwrap();
    insert_line(&pool, id).await;
    assert!(matches!(
        delete(&pool, id).await.unwrap_err(),
        StoreError::InUse
    ));
    assert_eq!(list(&pool).await.unwrap().len(), 1);
}

#[tokio::test]
async fn section_can_be_set_and_cleared() {
    let (_dir, pool) = database("ingredient-section").await;
    let id = create(&pool, "Tomato").await.unwrap();
    let section = crabbage_store::catalog::sections::create(&pool, "Produce")
        .await
        .unwrap();
    assert!(set_section(&pool, id, Some(section)).await.unwrap());
    assert_eq!(
        list(&pool).await.unwrap()[0].store_section_id,
        Some(section)
    );
    assert!(set_section(&pool, id, None).await.unwrap());
    assert_eq!(list(&pool).await.unwrap()[0].store_section_id, None);
    assert!(!set_section(&pool, 999, None).await.unwrap());
}

#[tokio::test]
async fn pending_ingredients_are_listed_until_approved() {
    let (_dir, pool) = database("ingredient-review").await;
    sqlx::query("INSERT INTO ingredients (name, pending_review) VALUES ('Okra', 1)")
        .execute(&pool)
        .await
        .unwrap();
    let pending = list_pending(&pool).await.unwrap();
    assert_eq!(pending.len(), 1);
    assert!(pending[0].pending_review);
    assert!(approve(&pool, pending[0].id).await.unwrap());
    assert_eq!(list_pending(&pool).await.unwrap().len(), 0);
    assert!(!approve(&pool, 999).await.unwrap());
}

#[expect(
    clippy::unwrap_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn insert_line(pool: &SqlitePool, ingredient: i64) {
    let user: i64 = sqlx::query_scalar(
        "INSERT INTO users (subject, name) VALUES ('cook', 'Cook') RETURNING id",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    let recipe: i64 = sqlx::query_scalar(
        "INSERT INTO recipes (owner_id, title, servings, created_at) \
         VALUES (?, 'Soup', 2, 0) RETURNING id",
    )
    .bind(user)
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO recipe_lines (recipe_id, position, ingredient_id) VALUES (?, 0, ?)")
        .bind(recipe)
        .bind(ingredient)
        .execute(pool)
        .await
        .unwrap();
}
