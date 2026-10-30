//! Recipes are filed under categories: the categories are stored with the recipe, replaced on
//! update, unknown ids are ignored, and the list filters by category.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::catalog::categories::{Category, create as create_category};
use crabbage_store::recipes::{RecipeInput, create, get, list, update};
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
async fn titles(pool: &SqlitePool, category: Option<i64>) -> Vec<String> {
    list(pool, category)
        .await
        .expect("list reads")
        .into_iter()
        .map(|summary| summary.title)
        .collect()
}

#[tokio::test]
async fn a_recipe_stores_its_categories_by_name_and_ignores_unknown_ids() {
    let (_dir, pool) = database("recipe-categories-store").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    let dinner = create_category(&pool, "Dinner").await.unwrap();
    let breakfast = create_category(&pool, "Breakfast").await.unwrap();

    let id = create(
        &pool,
        owner,
        &recipe("Pancakes", vec![dinner, breakfast, breakfast, 999]),
        0,
    )
    .await
    .unwrap();

    let stored = get(&pool, id).await.unwrap().expect("the recipe exists");
    assert_eq!(
        stored.categories,
        [
            Category {
                id: breakfast,
                name: "Breakfast".to_owned()
            },
            Category {
                id: dinner,
                name: "Dinner".to_owned()
            },
        ]
    );
}

#[tokio::test]
async fn update_replaces_the_categories() {
    let (_dir, pool) = database("recipe-categories-update").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    let dinner = create_category(&pool, "Dinner").await.unwrap();
    let breakfast = create_category(&pool, "Breakfast").await.unwrap();
    let id = create(&pool, owner, &recipe("Pancakes", vec![dinner]), 0)
        .await
        .unwrap();

    assert!(
        update(&pool, id, &recipe("Pancakes", vec![breakfast]))
            .await
            .unwrap()
    );
    let ids: Vec<i64> = get(&pool, id)
        .await
        .unwrap()
        .expect("the recipe exists")
        .categories
        .iter()
        .map(|category| category.id)
        .collect();
    assert_eq!(ids, [breakfast]);

    assert!(
        update(&pool, id, &recipe("Pancakes", vec![]))
            .await
            .unwrap()
    );
    assert_eq!(
        get(&pool, id).await.unwrap().expect("exists").categories,
        Vec::new()
    );
}

#[tokio::test]
async fn the_list_filters_by_category_and_shows_all_without_one() {
    let (_dir, pool) = database("recipe-categories-list").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    let dinner = create_category(&pool, "Dinner").await.unwrap();
    let breakfast = create_category(&pool, "Breakfast").await.unwrap();
    create(&pool, owner, &recipe("Pancakes", vec![breakfast]), 0)
        .await
        .unwrap();
    create(&pool, owner, &recipe("Roast", vec![dinner, breakfast]), 0)
        .await
        .unwrap();
    create(&pool, owner, &recipe("Toast", vec![]), 0)
        .await
        .unwrap();

    assert_eq!(titles(&pool, Some(breakfast)).await, ["Pancakes", "Roast"]);
    assert_eq!(titles(&pool, Some(dinner)).await, ["Roast"]);
    assert_eq!(titles(&pool, Some(999)).await, Vec::<String>::new());
    assert_eq!(titles(&pool, None).await, ["Pancakes", "Roast", "Toast"]);
}
