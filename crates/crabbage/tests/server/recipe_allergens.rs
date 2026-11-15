//! The recipe page and cook mode name the allergens of a recipe's ingredients, and say nothing
//! for a recipe without any.

use axum::http::StatusCode;
use crabbage_domain::allergens::Allergen;
use crabbage_store::catalog::allergens::set_allergens;
use crabbage_store::catalog::ingredients;
use crabbage_store::recipes::{LineInput, RecipeInput, StepInput, create};

use crate::common::{Harness, body_text, sign_in_as};

async fn page(harness: &Harness, token: &str, path: &str) -> String {
    let response = harness.get(path, Some(token)).await;
    assert_eq!(response.status(), StatusCode::OK, "{path}");
    body_text(response).await
}

fn recipe(title: &str, ingredient: &str) -> RecipeInput {
    RecipeInput {
        title: title.to_owned(),
        servings: 2,
        lines: vec![LineInput {
            amount: None,
            unit: None,
            ingredient: Some(ingredient.to_owned()),
            note: None,
            section_id: None,
            allergens: Vec::new(),
        }],
        steps: vec![StepInput {
            text: "Mix.".to_owned(),
            duration_seconds: None,
            line_positions: vec![0],
        }],
        category_ids: Vec::new(),
        meal_types: Vec::new(),
        warnings: Vec::new(),
    }
}

#[tokio::test]
async fn the_recipe_page_and_cook_mode_name_the_allergens() {
    let harness = Harness::start("recipe-allergens").await;
    let token = sign_in_as(&harness, "allergen-cook", &[]).await;
    let owner: i64 = sqlx::query_scalar("SELECT id FROM users WHERE subject = 'allergen-cook'")
        .fetch_one(&harness.pool)
        .await
        .expect("user exists");
    let pesto = ingredients::create(&harness.pool, "Pesto")
        .await
        .expect("ingredient created");
    set_allergens(&harness.pool, pesto, &[Allergen::TreeNuts, Allergen::Milk])
        .await
        .expect("allergens set");
    let pasta = create(&harness.pool, owner, &recipe("Pasta", "Pesto"), 0)
        .await
        .expect("recipe created");
    let salad = create(&harness.pool, owner, &recipe("Salad", "Basil"), 0)
        .await
        .expect("recipe created");

    let expected = "<strong>Contains:</strong> Milk, Tree nuts</p>";
    for path in [
        format!("/recipes/{pasta}"),
        format!("/recipes/{pasta}/cook"),
    ] {
        let body = page(&harness, &token, &path).await;
        assert!(
            body.contains("class=\"recipe-allergens\""),
            "{path}: {body}"
        );
        assert!(body.replace("<!>", "").contains(expected), "{path}: {body}");
    }
    for path in [
        format!("/recipes/{salad}"),
        format!("/recipes/{salad}/cook"),
    ] {
        let body = page(&harness, &token, &path).await;
        assert!(!body.contains("recipe-allergens"), "{path}: {body}");
    }
}
