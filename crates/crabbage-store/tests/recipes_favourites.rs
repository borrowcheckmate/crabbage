//! Favourites: each user has their own, and the list filters by them, alone or with a category.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::catalog::categories::create as create_category;
use crabbage_store::recipes::{RecipeInput, create, get, list, set_favourite};
use crabbage_store::users::upsert_user;
use sqlx::sqlite::SqlitePool;

fn recipe(title: &str, category_ids: Vec<i64>) -> RecipeInput {
    RecipeInput {
        title: title.to_owned(),
        servings: 2,
        lines: vec![],
        steps: vec![],
        category_ids,
    }
}

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn titles(
    pool: &SqlitePool,
    category: Option<i64>,
    favourites_only: bool,
    viewer: i64,
) -> Vec<String> {
    list(pool, category, favourites_only, viewer)
        .await
        .expect("list reads")
        .into_iter()
        .map(|summary| summary.title)
        .collect()
}

#[tokio::test]
async fn a_favourite_belongs_to_the_user_who_set_it() {
    let (_dir, pool) = database("favourites-toggle").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    let other = upsert_user(&pool, "other", "Other").await.unwrap();
    let id = create(&pool, owner, &recipe("Soup", vec![]), 0)
        .await
        .unwrap();

    assert!(set_favourite(&pool, id, owner, true).await.unwrap());
    assert!(set_favourite(&pool, id, owner, true).await.unwrap());
    assert!(
        get(&pool, id, owner)
            .await
            .unwrap()
            .expect("exists")
            .favourite
    );
    assert!(
        !get(&pool, id, other)
            .await
            .unwrap()
            .expect("exists")
            .favourite
    );

    assert!(set_favourite(&pool, id, owner, false).await.unwrap());
    assert!(
        !get(&pool, id, owner)
            .await
            .unwrap()
            .expect("exists")
            .favourite
    );
}

#[tokio::test]
async fn favouriting_a_missing_recipe_changes_nothing() {
    let (_dir, pool) = database("favourites-missing").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    assert!(!set_favourite(&pool, 999, owner, true).await.unwrap());
    assert!(!set_favourite(&pool, 999, owner, false).await.unwrap());
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM favourites")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn the_list_filters_by_the_viewers_favourites_with_and_without_a_category() {
    let (_dir, pool) = database("favourites-filter").await;
    let viewer = upsert_user(&pool, "viewer", "Viewer").await.unwrap();
    let friend = upsert_user(&pool, "friend", "Friend").await.unwrap();
    let breakfast = create_category(&pool, "Breakfast").await.unwrap();
    let dinner = create_category(&pool, "Dinner").await.unwrap();
    let pancakes = create(&pool, viewer, &recipe("Pancakes", vec![breakfast]), 0)
        .await
        .unwrap();
    let roast = create(&pool, viewer, &recipe("Roast", vec![dinner]), 0)
        .await
        .unwrap();
    let toast = create(&pool, viewer, &recipe("Toast", vec![breakfast]), 0)
        .await
        .unwrap();
    set_favourite(&pool, pancakes, viewer, true).await.unwrap();
    set_favourite(&pool, roast, viewer, true).await.unwrap();
    set_favourite(&pool, toast, friend, true).await.unwrap();

    assert_eq!(
        titles(&pool, None, true, viewer).await,
        ["Pancakes", "Roast"]
    );
    assert_eq!(
        titles(&pool, Some(breakfast), true, viewer).await,
        ["Pancakes"]
    );
    assert_eq!(
        titles(&pool, Some(breakfast), false, viewer).await,
        ["Pancakes", "Toast"]
    );
    assert_eq!(titles(&pool, None, true, friend).await, ["Toast"]);
    assert_eq!(
        titles(&pool, None, false, viewer).await,
        ["Pancakes", "Roast", "Toast"]
    );
}
