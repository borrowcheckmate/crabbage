//! Search: a case-insensitive part of a title, of an ingredient's name or of its alternative names,
//! with `%`, `_` and `\` taken literally, combined with the category and favourites filters.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::catalog::categories::create as create_category;
use crabbage_store::catalog::ingredients::{add_alias, create as create_ingredient};
use crabbage_store::recipes::{LineInput, RecipeInput, create, list, set_favourite};
use crabbage_store::users::upsert_user;
use sqlx::sqlite::SqlitePool;

fn recipe(title: &str, ingredients: &[&str], category_ids: Vec<i64>) -> RecipeInput {
    RecipeInput {
        title: title.to_owned(),
        servings: 2,
        lines: ingredients
            .iter()
            .copied()
            .map(|name| LineInput {
                amount: None,
                unit: None,
                ingredient: Some(name.to_owned()),
                note: None,
            })
            .collect(),
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
    search: Option<&str>,
    category: Option<i64>,
    favourites_only: bool,
    viewer: i64,
) -> Vec<String> {
    list(pool, category, favourites_only, search, viewer)
        .await
        .expect("list reads")
        .into_iter()
        .map(|summary| summary.title)
        .collect()
}

#[tokio::test]
async fn search_matches_titles_and_ingredient_names_ignoring_case() {
    let (_dir, pool) = database("search-basic").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    create(&pool, owner, &recipe("Tomato Soup", &["Onion"], vec![]), 0)
        .await
        .unwrap();
    create(
        &pool,
        owner,
        &recipe("Pancakes", &["Flour", "Egg"], vec![]),
        0,
    )
    .await
    .unwrap();
    create(&pool, owner, &recipe("Roast", &["Pork"], vec![]), 0)
        .await
        .unwrap();

    assert_eq!(
        titles(&pool, Some("tomato"), None, false, owner).await,
        ["Tomato Soup"]
    );
    assert_eq!(
        titles(&pool, Some("FLOUR"), None, false, owner).await,
        ["Pancakes"]
    );
    assert_eq!(
        titles(&pool, Some("onion"), None, false, owner).await,
        ["Tomato Soup"]
    );
    assert_eq!(
        titles(&pool, Some("oa"), None, false, owner).await,
        ["Roast"]
    );
    assert_eq!(
        titles(&pool, Some("zzz"), None, false, owner).await,
        Vec::<String>::new()
    );
}

#[tokio::test]
async fn search_matches_alternative_names_of_ingredients() {
    let (_dir, pool) = database("search-alias").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    let coriander = create_ingredient(&pool, "Coriander").await.unwrap();
    add_alias(&pool, coriander, "Cilantro").await.unwrap();
    create(&pool, owner, &recipe("Salsa", &["Coriander"], vec![]), 0)
        .await
        .unwrap();
    create(&pool, owner, &recipe("Rice", &["Rice"], vec![]), 0)
        .await
        .unwrap();

    assert_eq!(
        titles(&pool, Some("cilantro"), None, false, owner).await,
        ["Salsa"]
    );
}

#[tokio::test]
async fn percent_underscore_and_backslash_in_a_search_are_literal() {
    let (_dir, pool) = database("search-literal").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    for title in ["Rye 100% bread", "Bread_roll", "Bread roll", "Half\\Half"] {
        create(&pool, owner, &recipe(title, &[], vec![]), 0)
            .await
            .unwrap();
    }

    assert_eq!(
        titles(&pool, Some("%"), None, false, owner).await,
        ["Rye 100% bread"]
    );
    assert_eq!(
        titles(&pool, Some("d_r"), None, false, owner).await,
        ["Bread_roll"]
    );
    assert_eq!(
        titles(&pool, Some("\\"), None, false, owner).await,
        ["Half\\Half"]
    );
}

#[tokio::test]
async fn a_blank_search_is_no_search() {
    let (_dir, pool) = database("search-blank").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    create(&pool, owner, &recipe("Pancakes", &["Flour"], vec![]), 0)
        .await
        .unwrap();
    create(&pool, owner, &recipe("Roast", &["Pork"], vec![]), 0)
        .await
        .unwrap();

    assert_eq!(
        titles(&pool, None, None, false, owner).await,
        ["Pancakes", "Roast"]
    );
    assert_eq!(
        titles(&pool, Some("   "), None, false, owner).await,
        ["Pancakes", "Roast"]
    );
}

#[tokio::test]
async fn search_combines_with_the_category_and_favourites_filters() {
    let (_dir, pool) = database("search-filters").await;
    let viewer = upsert_user(&pool, "viewer", "Viewer").await.unwrap();
    let friend = upsert_user(&pool, "friend", "Friend").await.unwrap();
    let breakfast = create_category(&pool, "Breakfast").await.unwrap();
    let dinner = create_category(&pool, "Dinner").await.unwrap();
    let pancakes = create(
        &pool,
        viewer,
        &recipe("Pancakes", &["Flour"], vec![breakfast]),
        0,
    )
    .await
    .unwrap();
    let toast = create(
        &pool,
        viewer,
        &recipe("Toast", &["Bread"], vec![breakfast]),
        0,
    )
    .await
    .unwrap();
    let roast = create(&pool, viewer, &recipe("Roast", &["Pork"], vec![dinner]), 0)
        .await
        .unwrap();
    set_favourite(&pool, pancakes, viewer, true).await.unwrap();
    set_favourite(&pool, roast, viewer, true).await.unwrap();
    set_favourite(&pool, toast, friend, true).await.unwrap();

    assert_eq!(
        titles(&pool, Some("flour"), Some(breakfast), false, viewer).await,
        ["Pancakes"]
    );
    assert_eq!(
        titles(&pool, Some("pork"), Some(breakfast), false, viewer).await,
        Vec::<String>::new()
    );
    assert_eq!(
        titles(&pool, Some("pork"), Some(dinner), true, viewer).await,
        ["Roast"]
    );
    assert_eq!(
        titles(&pool, Some("bread"), None, true, viewer).await,
        Vec::<String>::new()
    );
    assert_eq!(
        titles(&pool, Some("bread"), None, true, friend).await,
        ["Toast"]
    );
    assert_eq!(
        titles(&pool, Some("a"), Some(breakfast), false, viewer).await,
        ["Pancakes", "Toast"]
    );
}
