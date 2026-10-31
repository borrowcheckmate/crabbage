//! The shopping list: recipes add scaled lines, manual items are added, edited and removed,
//! checked items sort last, clearing removes checked items, and every change stays in one household.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_domain::scaling::Servings;
use crabbage_store::StoreError;
use crabbage_store::planner::{PlanEntry, remove_entry, set_entry};
use crabbage_store::recipes::{LineInput, RecipeInput, create as create_recipe};
use crabbage_store::shopping::{
    ShoppingItem, add_plan, add_recipe, add_text, clear_checked, edit_text, list, remove,
    set_checked,
};
use crabbage_store::users::upsert_user;
use sqlx::sqlite::SqlitePool;

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
const fn servings(count: u32) -> Servings {
    Servings::new(count).expect("servings are positive")
}

#[expect(
    clippy::unwrap_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn household(pool: &SqlitePool, group: &str) -> i64 {
    sqlx::query_scalar("INSERT INTO households (group_name) VALUES (?) RETURNING id")
        .bind(group)
        .fetch_one(pool)
        .await
        .unwrap()
}

fn line(
    amount: Option<f64>,
    unit: Option<&str>,
    ingredient: Option<&str>,
    note: Option<&str>,
) -> LineInput {
    LineInput {
        amount,
        unit: unit.map(str::to_owned),
        ingredient: ingredient.map(str::to_owned),
        note: note.map(str::to_owned),
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn recipe(pool: &SqlitePool, lines: Vec<LineInput>) -> i64 {
    let owner = upsert_user(pool, "cook", "Cook").await.unwrap();
    let input = RecipeInput {
        title: "Bread".to_owned(),
        servings: 2,
        lines,
        steps: vec![],
        category_ids: vec![],
    };
    create_recipe(pool, owner, &input, 0).await.unwrap()
}

#[expect(
    clippy::unwrap_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn items(pool: &SqlitePool, household: i64) -> Vec<ShoppingItem> {
    list(pool, household).await.unwrap()
}

fn label(item: &ShoppingItem) -> String {
    item.ingredient
        .clone()
        .or_else(|| item.text.clone())
        .unwrap_or_default()
}

async fn labels(pool: &SqlitePool, household: i64) -> Vec<String> {
    items(pool, household).await.iter().map(label).collect()
}

#[tokio::test]
async fn adding_a_recipe_scales_its_lines_and_keeps_the_ones_with_content() {
    let (_dir, pool) = database("shopping-recipe").await;
    let home = household(&pool, "crabbage-household-home").await;
    let bread = recipe(
        &pool,
        vec![
            line(Some(200.0), Some("g"), Some("Flour"), Some("sifted")),
            line(Some(2.0), None, Some("Egg"), None),
            line(None, None, None, Some("grated lemon zest")),
            line(None, None, None, None),
        ],
    )
    .await;

    assert!(add_recipe(&pool, home, bread, servings(4)).await.unwrap());

    let items = items(&pool, home).await;
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].ingredient.as_deref(), Some("Flour"));
    assert_eq!(items[0].amount, Some(400.0));
    assert_eq!(items[0].unit.as_deref(), Some("g"));
    assert_eq!(items[0].text, None);
    assert_eq!(items[0].recipe_id, Some(bread));
    assert_eq!(items[1].ingredient.as_deref(), Some("Egg"));
    assert_eq!(items[1].amount, Some(4.0));
    assert_eq!(items[1].unit, None);
    assert_eq!(items[2].ingredient_id, None);
    assert_eq!(items[2].text.as_deref(), Some("grated lemon zest"));
    assert_eq!(items[2].recipe_id, Some(bread));
    assert!(items.iter().all(|item| !item.checked));
}

#[tokio::test]
async fn an_unknown_recipe_adds_nothing() {
    let (_dir, pool) = database("shopping-unknown-recipe").await;
    let home = household(&pool, "crabbage-household-home").await;

    assert!(!add_recipe(&pool, home, 999, servings(2)).await.unwrap());

    assert_eq!(labels(&pool, home).await, Vec::<String>::new());
}

#[tokio::test]
async fn manual_items_are_added_trimmed_and_edited_and_removed() {
    let (_dir, pool) = database("shopping-manual").await;
    let home = household(&pool, "crabbage-household-home").await;

    let id = add_text(&pool, home, "  Batteries ").await.unwrap();
    assert_eq!(labels(&pool, home).await, ["Batteries"]);

    assert!(edit_text(&pool, home, id, "AA batteries").await.unwrap());
    assert_eq!(labels(&pool, home).await, ["AA batteries"]);

    assert!(remove(&pool, home, id).await.unwrap());
    assert!(!remove(&pool, home, id).await.unwrap());
    assert_eq!(labels(&pool, home).await, Vec::<String>::new());
}

#[tokio::test]
async fn blank_text_is_refused_for_new_and_edited_items() {
    let (_dir, pool) = database("shopping-blank").await;
    let home = household(&pool, "crabbage-household-home").await;
    let id = add_text(&pool, home, "Milk").await.unwrap();

    assert!(matches!(
        add_text(&pool, home, "   ").await,
        Err(StoreError::Empty)
    ));
    assert!(matches!(
        edit_text(&pool, home, id, "\t").await,
        Err(StoreError::Empty)
    ));
    assert_eq!(labels(&pool, home).await, ["Milk"]);
}

#[tokio::test]
async fn editing_refuses_items_that_belong_to_an_ingredient() {
    let (_dir, pool) = database("shopping-edit-ingredient").await;
    let home = household(&pool, "crabbage-household-home").await;
    let bread = recipe(
        &pool,
        vec![line(Some(1.0), Some("kg"), Some("Flour"), None)],
    )
    .await;
    add_recipe(&pool, home, bread, servings(2)).await.unwrap();
    let id = items(&pool, home).await[0].id;

    assert!(!edit_text(&pool, home, id, "Rice").await.unwrap());

    assert_eq!(labels(&pool, home).await, ["Flour"]);
}

#[tokio::test]
async fn checked_items_sort_last_and_check_state_can_be_undone() {
    let (_dir, pool) = database("shopping-check").await;
    let home = household(&pool, "crabbage-household-home").await;
    let first = add_text(&pool, home, "Milk").await.unwrap();
    let second = add_text(&pool, home, "Eggs").await.unwrap();
    add_text(&pool, home, "Salt").await.unwrap();

    assert_eq!(set_checked(&pool, home, &[first], true).await.unwrap(), 1);
    assert_eq!(labels(&pool, home).await, ["Eggs", "Salt", "Milk"]);
    assert!(items(&pool, home).await[2].checked);

    assert_eq!(
        set_checked(&pool, home, &[first, second], false)
            .await
            .unwrap(),
        2
    );
    assert_eq!(labels(&pool, home).await, ["Milk", "Eggs", "Salt"]);
}

#[tokio::test]
async fn clearing_removes_only_the_checked_items() {
    let (_dir, pool) = database("shopping-clear").await;
    let home = household(&pool, "crabbage-household-home").await;
    let milk = add_text(&pool, home, "Milk").await.unwrap();
    add_text(&pool, home, "Eggs").await.unwrap();
    set_checked(&pool, home, &[milk], true).await.unwrap();

    assert_eq!(clear_checked(&pool, home).await.unwrap(), 1);

    assert_eq!(labels(&pool, home).await, ["Eggs"]);
    assert_eq!(clear_checked(&pool, home).await.unwrap(), 0);
}

#[tokio::test]
async fn another_household_cannot_see_or_change_the_list() {
    let (_dir, pool) = database("shopping-households").await;
    let home = household(&pool, "crabbage-household-home").await;
    let other = household(&pool, "crabbage-household-other").await;
    let milk = add_text(&pool, home, "Milk").await.unwrap();

    assert_eq!(labels(&pool, other).await, Vec::<String>::new());
    assert!(!edit_text(&pool, other, milk, "Oat milk").await.unwrap());
    assert!(!remove(&pool, other, milk).await.unwrap());
    assert_eq!(set_checked(&pool, other, &[milk], true).await.unwrap(), 0);
    assert_eq!(clear_checked(&pool, other).await.unwrap(), 0);

    assert_eq!(labels(&pool, home).await, ["Milk"]);
    assert!(!items(&pool, home).await[0].checked);
}

#[tokio::test]
async fn deleting_a_household_removes_its_items() {
    let (_dir, pool) = database("shopping-delete-household").await;
    let home = household(&pool, "crabbage-household-home").await;
    add_text(&pool, home, "Milk").await.unwrap();

    sqlx::query("DELETE FROM households WHERE id = ?")
        .bind(home)
        .execute(&pool)
        .await
        .unwrap();

    let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM shopping_items")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(left, 0);
}

#[expect(
    clippy::unwrap_used,
    reason = "test setup stops on an invalid plan entry"
)]
async fn set_plan(pool: &SqlitePool, home: i64, date: &str, recipe_id: Option<i64>, servings: i64) {
    set_entry(
        pool,
        home,
        &PlanEntry {
            date: date.to_owned(),
            recipe_id,
            servings,
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn adding_a_plan_scales_non_rest_days_once() {
    let (_dir, pool) = database("shopping-plan-add").await;
    let home = household(&pool, "crabbage-household-home").await;
    let bread = recipe(
        &pool,
        vec![line(Some(100.0), Some("g"), Some("Butter"), None)],
    )
    .await;
    set_plan(&pool, home, "2024-02-26", Some(bread), 4).await;
    set_plan(&pool, home, "2024-02-27", None, 4).await;
    set_plan(&pool, home, "2024-02-28", Some(bread), 2).await;

    assert_eq!(
        add_plan(&pool, home, "2024-02-26", "2024-03-03")
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        add_plan(&pool, home, "2024-02-26", "2024-03-03")
            .await
            .unwrap(),
        0
    );
    let items = items(&pool, home).await;
    assert_eq!(
        items
            .iter()
            .map(|item| item.amount.unwrap())
            .collect::<Vec<_>>(),
        [200.0, 100.0]
    );
    assert!(items.iter().all(|item| item.plan_entry_id.is_some()));
}

#[tokio::test]
async fn changing_or_removing_a_listed_entry_replaces_unchecked_items_and_keeps_checked() {
    let (_dir, pool) = database("shopping-plan-change").await;
    let home = household(&pool, "crabbage-household-home").await;
    let first = recipe(
        &pool,
        vec![line(Some(100.0), Some("g"), Some("Butter"), None)],
    )
    .await;
    let second = recipe(
        &pool,
        vec![line(Some(2.0), Some("tbsp"), Some("Milk"), None)],
    )
    .await;
    set_plan(&pool, home, "2024-02-26", Some(first), 2).await;
    add_plan(&pool, home, "2024-02-26", "2024-02-26")
        .await
        .unwrap();
    let checked = items(&pool, home).await[0].id;
    set_checked(&pool, home, &[checked], true).await.unwrap();
    set_plan(&pool, home, "2024-02-26", Some(second), 4).await;
    let listed = items(&pool, home).await;
    assert_eq!(listed.len(), 2);
    assert!(listed.iter().any(|item| item.id == checked && item.checked));
    assert!(
        listed
            .iter()
            .any(|item| item.ingredient.as_deref() == Some("Milk") && item.amount == Some(4.0))
    );

    set_plan(&pool, home, "2024-02-26", None, 4).await;
    assert_eq!(items(&pool, home).await.len(), 1);
    assert!(remove_entry(&pool, home, "2024-02-26").await.unwrap());
    assert_eq!(items(&pool, home).await.len(), 1);
}
