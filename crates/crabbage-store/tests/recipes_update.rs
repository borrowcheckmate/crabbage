//! Updating a recipe replaces its title, servings, lines and steps. A missing recipe is not
//! changed, and a bad step link changes nothing.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::StoreError;
use crabbage_store::recipes::{LineInput, RecipeInput, StepInput, create, get, update};
use crabbage_store::users::upsert_user;

fn plain_line(ingredient: &str) -> LineInput {
    LineInput {
        amount: Some(1.0),
        unit: Some("piece".to_owned()),
        ingredient: Some(ingredient.to_owned()),
        note: None,
    }
}

fn input(title: &str, lines: Vec<LineInput>, steps: Vec<StepInput>) -> RecipeInput {
    RecipeInput {
        title: title.to_owned(),
        servings: 2,
        lines,
        steps,
    }
}

#[tokio::test]
async fn update_replaces_the_title_lines_and_steps() {
    let (_dir, pool) = database("recipes-update").await;
    let owner = upsert_user(&pool, "cook", "Cook").await.unwrap();
    let first = input(
        "Soup",
        vec![plain_line("Onion"), plain_line("Carrot")],
        vec![StepInput {
            text: "Chop".to_owned(),
            duration_seconds: None,
            line_positions: vec![0, 1],
        }],
    );
    let id = create(&pool, owner, &first, 0).await.unwrap();

    let second = RecipeInput {
        servings: 6,
        ..input(
            "Better soup",
            vec![plain_line("Leek")],
            vec![StepInput {
                text: "Simmer".to_owned(),
                duration_seconds: Some(600),
                line_positions: vec![0],
            }],
        )
    };
    assert!(update(&pool, id, &second).await.unwrap());

    let recipe = get(&pool, id).await.unwrap().expect("the recipe exists");
    assert_eq!(recipe.title, "Better soup");
    assert_eq!(recipe.servings, 6);
    let lines: Vec<Option<&str>> = recipe
        .lines
        .iter()
        .map(|line| line.ingredient.as_deref())
        .collect();
    assert_eq!(lines, [Some("Leek")]);
    assert_eq!(recipe.steps.len(), 1);
    assert_eq!(recipe.steps[0].text, "Simmer");
    assert_eq!(recipe.steps[0].line_ids, [recipe.lines[0].id]);
    let stale_links: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM step_lines")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stale_links, 1, "the replaced step's links are gone");
}

#[tokio::test]
async fn update_of_a_missing_recipe_changes_nothing() {
    let (_dir, pool) = database("recipes-update-missing").await;
    assert!(
        !update(&pool, 7, &input("Soup", vec![], vec![]))
            .await
            .unwrap()
    );
    let recipes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM recipes")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(recipes, 0);
}

#[tokio::test]
async fn a_bad_step_link_on_update_keeps_the_recipe_as_it_was() {
    let (_dir, pool) = database("recipes-update-bad-link").await;
    let owner = upsert_user(&pool, "cook", "Cook").await.unwrap();
    let id = create(
        &pool,
        owner,
        &input("Soup", vec![plain_line("Onion")], vec![]),
        0,
    )
    .await
    .unwrap();

    let broken = input(
        "Broken",
        vec![],
        vec![StepInput {
            text: "Stir".to_owned(),
            duration_seconds: None,
            line_positions: vec![3],
        }],
    );
    assert!(matches!(
        update(&pool, id, &broken).await,
        Err(StoreError::UnknownLine)
    ));

    let recipe = get(&pool, id).await.unwrap().expect("the recipe exists");
    assert_eq!(recipe.title, "Soup");
    assert_eq!(recipe.lines.len(), 1);
}
