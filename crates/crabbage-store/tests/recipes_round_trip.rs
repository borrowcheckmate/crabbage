//! A recipe round-trips with its lines and steps in order, the step links to its lines, an unknown
//! ingredient waits for review, and a known name or alias resolves to its ingredient.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::catalog::ingredients::{add_alias, create as create_ingredient, list_pending};
use crabbage_store::recipes::{LineInput, RecipeInput, StepInput, create, get, list};
use crabbage_store::users::upsert_user;

fn line(amount: f64, unit: &str, ingredient: &str, note: Option<&str>) -> LineInput {
    LineInput {
        amount: Some(amount),
        unit: Some(unit.to_owned()),
        ingredient: Some(ingredient.to_owned()),
        note: note.map(str::to_owned),
    }
}

fn pancakes(lines: Vec<LineInput>, steps: Vec<StepInput>) -> RecipeInput {
    RecipeInput {
        title: "Pancakes".to_owned(),
        servings: 4,
        lines,
        steps,
        category_ids: vec![],
    }
}

#[tokio::test]
async fn a_recipe_round_trips_with_lines_steps_and_links() {
    let (_dir, pool) = database("recipes-round-trip").await;
    let owner = upsert_user(&pool, "cook", "Cook").await.unwrap();
    create_ingredient(&pool, "Flour").await.unwrap();
    let input = pancakes(
        vec![
            line(250.0, "g", "Flour", Some("sifted")),
            LineInput {
                amount: None,
                unit: Some("to taste".to_owned()),
                ingredient: None,
                note: None,
            },
        ],
        vec![
            StepInput {
                text: "Mix".to_owned(),
                duration_seconds: Some(120),
                line_positions: vec![0],
            },
            StepInput {
                text: "Fry".to_owned(),
                duration_seconds: None,
                line_positions: vec![0, 1],
            },
        ],
    );

    let id = create(&pool, owner, &input, 1_000).await.unwrap();

    let recipe = get(&pool, id, owner)
        .await
        .unwrap()
        .expect("the recipe exists");
    assert_eq!(recipe.title, "Pancakes");
    assert_eq!(recipe.servings, 4);
    assert_eq!(recipe.owner_id, owner);
    assert_eq!(recipe.owner_name, "Cook");
    assert_eq!(recipe.editors, Vec::new());
    let line_ids: Vec<i64> = recipe.lines.iter().map(|line| line.id).collect();
    assert_eq!(recipe.lines.len(), 2);
    assert_eq!(recipe.lines[0].amount, Some(250.0));
    assert_eq!(recipe.lines[0].unit.as_deref(), Some("g"));
    assert_eq!(recipe.lines[0].ingredient.as_deref(), Some("Flour"));
    assert_eq!(recipe.lines[0].note.as_deref(), Some("sifted"));
    assert_eq!(recipe.lines[1].ingredient, None);
    assert_eq!(recipe.steps.len(), 2);
    assert_eq!(recipe.steps[0].text, "Mix");
    assert_eq!(recipe.steps[0].duration_seconds, Some(120));
    assert_eq!(recipe.steps[0].line_ids, [line_ids[0]]);
    assert_eq!(recipe.steps[1].duration_seconds, None);
    assert_eq!(recipe.steps[1].line_ids, line_ids);
}

#[tokio::test]
async fn an_unknown_ingredient_is_created_pending_review() {
    let (_dir, pool) = database("recipes-pending").await;
    let owner = upsert_user(&pool, "cook", "Cook").await.unwrap();
    let input = pancakes(vec![line(1.0, "piece", "Dragonfruit", None)], vec![]);
    create(&pool, owner, &input, 0).await.unwrap();

    let pending: Vec<(String, bool)> =
        sqlx::query_as("SELECT name, pending_review FROM ingredients")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(pending, [("Dragonfruit".to_owned(), true)]);
}

#[tokio::test]
async fn names_and_aliases_resolve_case_insensitively() {
    let (_dir, pool) = database("recipes-aliases").await;
    let owner = upsert_user(&pool, "cook", "Cook").await.unwrap();
    let tomato = create_ingredient(&pool, "Tomato").await.unwrap();
    add_alias(&pool, tomato, "Pomodoro").await.unwrap();
    let input = pancakes(
        vec![
            line(2.0, "piece", "tomato", None),
            line(1.0, "piece", "POMODORO", None),
        ],
        vec![],
    );
    let id = create(&pool, owner, &input, 0).await.unwrap();

    let recipe = get(&pool, id, owner)
        .await
        .unwrap()
        .expect("the recipe exists");
    let names: Vec<Option<&str>> = recipe
        .lines
        .iter()
        .map(|line| line.ingredient.as_deref())
        .collect();
    assert_eq!(names, [Some("Tomato"), Some("Tomato")]);
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ingredients")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1, "no ingredient is created for known names");
}

#[tokio::test]
async fn a_missing_recipe_is_none() {
    let (_dir, pool) = database("recipes-missing").await;
    let owner = upsert_user(&pool, "cook", "Cook").await.unwrap();
    assert!(get(&pool, 42, owner).await.unwrap().is_none());
}

#[tokio::test]
async fn the_list_is_ordered_by_title() {
    let (_dir, pool) = database("recipes-list").await;
    let owner = upsert_user(&pool, "cook", "Cook").await.unwrap();
    for title in ["soup", "Bread", "Cake"] {
        let mut input = pancakes(vec![], vec![]);
        input.title = title.to_owned();
        create(&pool, owner, &input, 0).await.unwrap();
    }
    let titles: Vec<String> = list(&pool, None, false, None, owner)
        .await
        .unwrap()
        .into_iter()
        .map(|summary| summary.title)
        .collect();
    assert_eq!(titles, ["Bread", "Cake", "soup"]);
}

#[tokio::test]
async fn a_step_linking_a_missing_line_is_refused_and_nothing_is_saved() {
    let (_dir, pool) = database("recipes-unknown-line").await;
    let owner = upsert_user(&pool, "cook", "Cook").await.unwrap();
    let input = pancakes(
        vec![],
        vec![StepInput {
            text: "Mix".to_owned(),
            duration_seconds: None,
            line_positions: vec![0],
        }],
    );
    let result = create(&pool, owner, &input, 0).await;
    assert!(matches!(
        result,
        Err(crabbage_store::StoreError::UnknownLine)
    ));
    assert_eq!(
        list(&pool, None, false, None, owner).await.unwrap().len(),
        0
    );
}

#[tokio::test]
async fn a_new_ingredient_waits_for_review_with_a_capital_first_letter() {
    let (_dir, pool) = database("recipes-capital").await;
    let owner = upsert_user(&pool, "cook", "Cook").await.unwrap();
    let input = pancakes(vec![line(2.0, "tbsp", "ahornsiroop", None)], vec![]);
    create(&pool, owner, &input, 0).await.unwrap();
    let pending: Vec<String> = list_pending(&pool)
        .await
        .unwrap()
        .into_iter()
        .map(|ingredient| ingredient.name)
        .collect();
    assert_eq!(pending, ["Ahornsiroop"]);
}
