//! The planner picks only recipes of its meal types or of none, from categories, from all recipes
//! and in the recipe picker, and no meal types means any recipe.

use crate::common::database;
use crabbage_domain::meal_types::MealType;
use crabbage_store::planner::{recipe_ids, recipe_titles, recipes_in_categories};
use crabbage_store::recipes::{RecipeInput, create};
use crabbage_store::users::upsert_user;

#[tokio::test]
async fn only_recipes_of_the_planned_meal_types_or_of_none_qualify() {
    let (_dir, pool) = database("planner-meal-types").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    let category: i64 =
        sqlx::query_scalar("INSERT INTO categories (name) VALUES ('Quick') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let mut ids = Vec::new();
    for (title, meal_types) in [
        ("Pancakes", vec![MealType::Breakfast]),
        ("Roast", vec![MealType::Dinner, MealType::Lunch]),
        ("Soup", Vec::new()),
    ] {
        let input = RecipeInput {
            title: title.to_owned(),
            servings: 2,
            lines: Vec::new(),
            steps: Vec::new(),
            category_ids: vec![category],
            meal_types,
        };
        ids.push(create(&pool, owner, &input, 0).await.unwrap());
    }
    let [pancakes, roast, soup] = ids[..] else {
        panic!("three recipes");
    };

    assert_eq!(
        recipe_ids(&pool, &[MealType::Dinner]).await.unwrap(),
        [roast, soup]
    );
    assert_eq!(
        recipe_ids(&pool, &[MealType::Breakfast, MealType::Lunch])
            .await
            .unwrap(),
        [pancakes, roast, soup]
    );
    assert_eq!(recipe_ids(&pool, &[MealType::Drink]).await.unwrap(), [soup]);
    assert_eq!(
        recipe_ids(&pool, &[]).await.unwrap(),
        [pancakes, roast, soup]
    );
    assert_eq!(
        recipes_in_categories(&pool, &[category], &[MealType::Dinner])
            .await
            .unwrap(),
        [(category, roast), (category, soup)]
    );
    assert_eq!(
        recipe_titles(&pool, &[MealType::Breakfast]).await.unwrap(),
        [(pancakes, "Pancakes".to_owned()), (soup, "Soup".to_owned())]
    );
}
