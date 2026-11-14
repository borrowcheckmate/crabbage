//! A recipe's allergens are those of its ingredients, each once and in the fixed order.

use crate::common::database;
use crabbage_domain::allergens::Allergen;
use crabbage_store::catalog::allergens::set_allergens;
use crabbage_store::catalog::ingredients::create as create_ingredient;
use crabbage_store::recipes::{LineInput, RecipeInput, create, get};
use crabbage_store::users::upsert_user;

fn line(ingredient: &str) -> LineInput {
    LineInput {
        amount: None,
        unit: None,
        ingredient: Some(ingredient.to_owned()),
        note: None,
        section_id: None,
    }
}

#[tokio::test]
async fn a_recipe_has_the_allergens_of_its_ingredients_once_in_order() {
    let (_dir, pool) = database("recipe-allergens").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    let pesto = create_ingredient(&pool, "Pesto").await.unwrap();
    let flour = create_ingredient(&pool, "Flour").await.unwrap();
    let cheese = create_ingredient(&pool, "Cheese").await.unwrap();
    set_allergens(&pool, pesto, &[Allergen::TreeNuts, Allergen::Milk])
        .await
        .unwrap();
    set_allergens(&pool, flour, &[Allergen::Gluten])
        .await
        .unwrap();
    set_allergens(&pool, cheese, &[Allergen::Milk])
        .await
        .unwrap();
    let input = RecipeInput {
        title: "Pasta".to_owned(),
        servings: 2,
        lines: vec![line("Pesto"), line("Cheese"), line("Flour"), line("Basil")],
        steps: Vec::new(),
        category_ids: Vec::new(),
        meal_types: Vec::new(),
    };
    let id = create(&pool, owner, &input, 0).await.unwrap();
    let plain = create(
        &pool,
        owner,
        &RecipeInput {
            title: "Salad".to_owned(),
            lines: vec![line("Basil")],
            ..input
        },
        0,
    )
    .await
    .unwrap();

    let recipe = get(&pool, id, owner).await.unwrap().unwrap();
    assert_eq!(
        recipe.allergens,
        [Allergen::Gluten, Allergen::Milk, Allergen::TreeNuts]
    );
    let salad = get(&pool, plain, owner).await.unwrap().unwrap();
    assert_eq!(salad.allergens, Vec::<Allergen>::new());
}
