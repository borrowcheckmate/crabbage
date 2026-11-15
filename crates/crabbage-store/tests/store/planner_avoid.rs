//! A household's avoid list is saved and replaced, and the recipes that conflict with it are those
//! with an avoided allergen in an ingredient or an avoided warning.

use crabbage_domain::allergens::Allergen;
use crabbage_domain::warnings::Warning;

use crate::common::database;
use crabbage_store::catalog::allergens::set_allergens;
use crabbage_store::catalog::ingredients::create as create_ingredient;
use crabbage_store::planner::{Avoid, avoid, conflicting_recipes, save_avoid};
use crabbage_store::recipes::{LineInput, RecipeInput, create};
use crabbage_store::users::upsert_user;

fn recipe(title: &str, ingredient: &str, warnings: Vec<Warning>) -> RecipeInput {
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
        steps: Vec::new(),
        category_ids: Vec::new(),
        meal_types: Vec::new(),
        warnings,
    }
}

#[tokio::test]
async fn the_avoid_list_is_saved_and_replaced() {
    let (_dir, pool) = database("planner-avoid").await;
    let household: i64 =
        sqlx::query_scalar("INSERT INTO households (group_name) VALUES ('home') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(avoid(&pool, household).await.unwrap(), Avoid::default());

    let saved = Avoid {
        allergens: vec![Allergen::TreeNuts],
        warnings: vec![Warning::Alcohol],
    };
    save_avoid(&pool, household, &saved).await.unwrap();
    assert_eq!(avoid(&pool, household).await.unwrap(), saved);
    save_avoid(&pool, household, &Avoid::default())
        .await
        .unwrap();
    assert_eq!(avoid(&pool, household).await.unwrap(), Avoid::default());
}

#[tokio::test]
async fn recipes_conflict_by_ingredient_allergen_or_warning() {
    let (_dir, pool) = database("planner-avoid-conflicts").await;
    let owner = upsert_user(&pool, "cook", "Cook").await.unwrap();
    let pesto = create_ingredient(&pool, "Pesto").await.unwrap();
    set_allergens(&pool, pesto, &[Allergen::TreeNuts])
        .await
        .unwrap();
    let pasta = create(&pool, owner, &recipe("Pasta", "Pesto", Vec::new()), 0)
        .await
        .unwrap();
    let stew = create(
        &pool,
        owner,
        &recipe("Stew", "Beef", vec![Warning::Alcohol]),
        0,
    )
    .await
    .unwrap();
    create(&pool, owner, &recipe("Toast", "Bread", Vec::new()), 0)
        .await
        .unwrap();

    let both = Avoid {
        allergens: vec![Allergen::TreeNuts],
        warnings: vec![Warning::Alcohol],
    };
    assert_eq!(
        conflicting_recipes(&pool, &both).await.unwrap(),
        [pasta, stew]
    );
    let nuts = Avoid {
        allergens: vec![Allergen::TreeNuts],
        warnings: Vec::new(),
    };
    assert_eq!(conflicting_recipes(&pool, &nuts).await.unwrap(), [pasta]);
    let none = conflicting_recipes(&pool, &Avoid::default()).await.unwrap();
    assert_eq!(none, Vec::<i64>::new());
}
