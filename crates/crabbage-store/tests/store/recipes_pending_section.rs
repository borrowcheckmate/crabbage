//! An ingredient created for review by a recipe line gets the store section that line names. A
//! section that does not exist is ignored, and an existing ingredient keeps its own section.

use crate::common::database;
use crabbage_store::catalog::ingredients::{
    create as create_ingredient, list_pending, set_section,
};
use crabbage_store::catalog::sections;
use crabbage_store::recipes::{LineInput, RecipeInput, create};
use crabbage_store::users::upsert_user;

fn line(ingredient: &str, section_id: Option<i64>) -> LineInput {
    LineInput {
        amount: Some(2.0),
        unit: Some("tbsp".to_owned()),
        ingredient: Some(ingredient.to_owned()),
        note: None,
        section_id,
        allergens: Vec::new(),
    }
}

fn recipe(lines: Vec<LineInput>) -> RecipeInput {
    RecipeInput {
        title: "Pancakes".to_owned(),
        servings: 2,
        lines,
        steps: Vec::new(),
        category_ids: Vec::new(),
        meal_types: Vec::new(),
        warnings: Vec::new(),
    }
}

#[tokio::test]
async fn a_new_ingredient_takes_the_section_of_its_line_and_an_unknown_section_is_ignored() {
    let (_dir, pool) = database("recipes-pending-section").await;
    let owner = upsert_user(&pool, "cook", "Cook").await.unwrap();
    let dairy = sections::create(&pool, "Dairy").await.unwrap();
    let input = recipe(vec![
        line("Ahornsiroop", Some(dairy)),
        line("Zout", Some(9999)),
        line("Vanille", None),
    ]);
    create(&pool, owner, &input, 0).await.unwrap();

    let pending = list_pending(&pool).await.unwrap();
    let section_of = |name: &str| {
        pending
            .iter()
            .find(|ingredient| ingredient.name == name)
            .and_then(|ingredient| ingredient.store_section_id)
    };
    assert_eq!(section_of("Ahornsiroop"), Some(dairy));
    assert_eq!(section_of("Zout"), None);
    assert_eq!(section_of("Vanille"), None);
}

#[tokio::test]
async fn an_existing_ingredient_keeps_its_section() {
    let (_dir, pool) = database("recipes-existing-section").await;
    let owner = upsert_user(&pool, "cook", "Cook").await.unwrap();
    let dairy = sections::create(&pool, "Dairy").await.unwrap();
    let bakery = sections::create(&pool, "Bakery").await.unwrap();
    let melk = create_ingredient(&pool, "Melk").await.unwrap();
    set_section(&pool, melk, Some(bakery)).await.unwrap();
    create(&pool, owner, &recipe(vec![line("Melk", Some(dairy))]), 0)
        .await
        .unwrap();

    let ingredients = crabbage_store::catalog::ingredients::list(&pool)
        .await
        .unwrap();
    let stored = ingredients
        .iter()
        .find(|ingredient| ingredient.id == melk)
        .unwrap();
    assert_eq!(stored.store_section_id, Some(bakery));
}
