//! Merging: recipe lines move to the target, aliases move, the source name becomes an alias, and
//! the source is deleted.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::catalog::ingredients::{add_alias, create, list, merge};
use sqlx::sqlite::SqlitePool;

#[expect(
    clippy::unwrap_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn line_ingredients(pool: &SqlitePool) -> Vec<i64> {
    sqlx::query_scalar("SELECT ingredient_id FROM recipe_lines ORDER BY id")
        .fetch_all(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn merge_repoints_recipe_lines_moves_aliases_and_deletes_the_source() {
    let (_dir, pool) = database("merge-repoint").await;
    let from = create(&pool, "Scallion").await.unwrap();
    let into = create(&pool, "Spring onion").await.unwrap();
    add_alias(&pool, from, "Green onion").await.unwrap();
    insert_recipe_with_lines(&pool, &[from, from]).await;

    assert!(merge(&pool, from, into).await.unwrap());

    assert_eq!(line_ingredients(&pool).await, [into, into]);
    let ingredients = list(&pool).await.unwrap();
    assert_eq!(ingredients.len(), 1);
    assert_eq!(ingredients[0].id, into);
    assert_eq!(ingredients[0].aliases, ["Green onion", "Scallion"]);
}

#[tokio::test]
async fn merge_needs_two_different_existing_ingredients() {
    let (_dir, pool) = database("merge-invalid").await;
    let id = create(&pool, "Scallion").await.unwrap();
    assert!(!merge(&pool, id, id).await.unwrap());
    assert!(!merge(&pool, id, 999).await.unwrap());
    assert!(!merge(&pool, 999, id).await.unwrap());
    assert_eq!(list(&pool).await.unwrap().len(), 1);
}

#[expect(
    clippy::unwrap_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn insert_recipe_with_lines(pool: &SqlitePool, ingredients: &[i64]) {
    let user: i64 = sqlx::query_scalar(
        "INSERT INTO users (subject, name) VALUES ('cook', 'Cook') RETURNING id",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    let recipe: i64 = sqlx::query_scalar(
        "INSERT INTO recipes (owner_id, title, servings, created_at) \
         VALUES (?, 'Salad', 2, 0) RETURNING id",
    )
    .bind(user)
    .fetch_one(pool)
    .await
    .unwrap();
    for (position, ingredient) in ingredients.iter().enumerate() {
        let position = i64::try_from(position).unwrap();
        sqlx::query(
            "INSERT INTO recipe_lines (recipe_id, position, ingredient_id) VALUES (?, ?, ?)",
        )
        .bind(recipe)
        .bind(position)
        .bind(ingredient)
        .execute(pool)
        .await
        .unwrap();
    }
}
