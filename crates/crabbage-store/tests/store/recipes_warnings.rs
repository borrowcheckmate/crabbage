//! A recipe keeps its dietary warnings once each and in order, and an update replaces them. An
//! ingredient created for review by a line gets that line's allergens; an existing one keeps its own.

use crabbage_domain::allergens::Allergen;
use crabbage_domain::warnings::Warning;

use crate::common::database;
use crabbage_store::catalog::allergens::set_allergens;
use crabbage_store::catalog::ingredients::{create as create_ingredient, list, list_pending};
use crabbage_store::recipes::{LineInput, RecipeInput, create, get, update};
use crabbage_store::users::upsert_user;

fn line(ingredient: &str, allergens: Vec<Allergen>) -> LineInput {
    LineInput {
        amount: None,
        unit: None,
        ingredient: Some(ingredient.to_owned()),
        note: None,
        section_id: None,
        allergens,
    }
}

fn recipe(lines: Vec<LineInput>, warnings: Vec<Warning>) -> RecipeInput {
    RecipeInput {
        title: "Tiramisu".to_owned(),
        servings: 4,
        lines,
        steps: Vec::new(),
        category_ids: Vec::new(),
        meal_types: Vec::new(),
        warnings,
    }
}

#[tokio::test]
async fn warnings_are_kept_once_in_order_and_replaced_on_update() {
    let (_dir, pool) = database("recipes-warnings").await;
    let owner = upsert_user(&pool, "cook", "Cook").await.unwrap();
    let input = recipe(
        Vec::new(),
        vec![Warning::Alcohol, Warning::RawEgg, Warning::Alcohol],
    );
    let id = create(&pool, owner, &input, 0).await.unwrap();
    let stored = get(&pool, id, owner).await.unwrap().unwrap();
    assert_eq!(stored.warnings, [Warning::RawEgg, Warning::Alcohol]);

    update(&pool, id, &recipe(Vec::new(), vec![Warning::Liver]))
        .await
        .unwrap();
    let stored = get(&pool, id, owner).await.unwrap().unwrap();
    assert_eq!(stored.warnings, [Warning::Liver]);
}

#[tokio::test]
async fn a_new_ingredient_takes_the_allergens_of_its_line_and_an_existing_one_keeps_its_own() {
    let (_dir, pool) = database("recipes-pending-allergens").await;
    let owner = upsert_user(&pool, "cook", "Cook").await.unwrap();
    let melk = create_ingredient(&pool, "Melk").await.unwrap();
    set_allergens(&pool, melk, &[Allergen::Milk]).await.unwrap();
    let lines = vec![
        line("Mascarpone", vec![Allergen::Milk]),
        line("Melk", vec![Allergen::Eggs]),
    ];
    create(&pool, owner, &recipe(lines, Vec::new()), 0)
        .await
        .unwrap();

    let pending = list_pending(&pool).await.unwrap();
    let mascarpone = pending
        .iter()
        .find(|ingredient| ingredient.name == "Mascarpone")
        .unwrap();
    assert_eq!(mascarpone.allergens, [Allergen::Milk]);
    let all = list(&pool).await.unwrap();
    let melk = all.iter().find(|ingredient| ingredient.id == melk).unwrap();
    assert_eq!(melk.allergens, [Allergen::Milk]);
}
