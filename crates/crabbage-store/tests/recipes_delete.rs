//! Deleting a recipe removes its lines, steps, step links, editors, ratings and favourites.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::recipes::{LineInput, RecipeInput, StepInput, add_editor, create, delete, get};
use crabbage_store::users::upsert_user;
use sqlx::sqlite::SqlitePool;

#[expect(
    clippy::unwrap_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn count(pool: &SqlitePool, sql: &'static str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

#[tokio::test]
async fn delete_removes_everything_that_belongs_to_the_recipe() {
    let (_dir, pool) = database("recipes-delete").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    let friend = upsert_user(&pool, "friend", "Friend").await.unwrap();
    let input = RecipeInput {
        title: "Soup".to_owned(),
        servings: 2,
        category_ids: vec![],
        lines: vec![LineInput {
            amount: Some(1.0),
            unit: Some("piece".to_owned()),
            ingredient: Some("Onion".to_owned()),
            note: None,
        }],
        steps: vec![StepInput {
            text: "Chop".to_owned(),
            duration_seconds: Some(60),
            line_positions: vec![0],
        }],
    };
    let id = create(&pool, owner, &input, 0).await.unwrap();
    add_editor(&pool, id, friend).await.unwrap();
    sqlx::query("INSERT INTO ratings (recipe_id, user_id, stars) VALUES (?, ?, 5)")
        .bind(id)
        .bind(friend)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO favourites (recipe_id, user_id) VALUES (?, ?)")
        .bind(id)
        .bind(friend)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(count(&pool, "SELECT COUNT(*) FROM recipe_lines").await, 1);
    assert_eq!(count(&pool, "SELECT COUNT(*) FROM steps").await, 1);
    assert_eq!(count(&pool, "SELECT COUNT(*) FROM step_lines").await, 1);

    assert!(delete(&pool, id).await.unwrap());

    assert!(get(&pool, id).await.unwrap().is_none());
    for sql in [
        "SELECT COUNT(*) FROM recipes",
        "SELECT COUNT(*) FROM recipe_lines",
        "SELECT COUNT(*) FROM steps",
        "SELECT COUNT(*) FROM step_lines",
        "SELECT COUNT(*) FROM recipe_editors",
        "SELECT COUNT(*) FROM ratings",
        "SELECT COUNT(*) FROM favourites",
    ] {
        assert_eq!(count(&pool, sql).await, 0, "{sql}");
    }
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM ingredients").await,
        1,
        "ingredients stay"
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM users").await,
        2,
        "users stay"
    );
}

#[tokio::test]
async fn delete_of_a_missing_recipe_is_false() {
    let (_dir, pool) = database("recipes-delete-missing").await;
    assert!(!delete(&pool, 5).await.unwrap());
}
