//! Recipes carry fixed meal types: stored once each in a fixed order, replaced on update, none for a
//! recipe without them, and the list filters by meal type.

use crate::common::database;
use crabbage_domain::meal_types::MealType;
use crabbage_store::recipes::{RecipeInput, create, get, list, update};
use crabbage_store::users::upsert_user;

fn recipe(title: &str, meal_types: Vec<MealType>) -> RecipeInput {
    RecipeInput {
        title: title.to_owned(),
        servings: 2,
        lines: Vec::new(),
        steps: Vec::new(),
        category_ids: Vec::new(),
        meal_types,
        warnings: Vec::new(),
    }
}

#[tokio::test]
async fn meal_types_are_stored_once_in_order_and_replaced_on_update() {
    let (_dir, pool) = database("recipe-meal-types-store").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    let id = create(
        &pool,
        owner,
        &recipe(
            "Pancakes",
            vec![MealType::Dinner, MealType::Breakfast, MealType::Dinner],
        ),
        0,
    )
    .await
    .unwrap();
    let stored = get(&pool, id, owner).await.unwrap().unwrap();
    assert_eq!(stored.meal_types, [MealType::Breakfast, MealType::Dinner]);

    update(&pool, id, &recipe("Pancakes", vec![MealType::Dessert]))
        .await
        .unwrap();
    let stored = get(&pool, id, owner).await.unwrap().unwrap();
    assert_eq!(stored.meal_types, [MealType::Dessert]);

    let plain = create(&pool, owner, &recipe("Soup", Vec::new()), 0)
        .await
        .unwrap();
    assert_eq!(
        get(&pool, plain, owner).await.unwrap().unwrap().meal_types,
        Vec::<MealType>::new()
    );
}

#[tokio::test]
async fn the_list_keeps_only_recipes_of_the_chosen_meal_type() {
    let (_dir, pool) = database("recipe-meal-types-list").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    create(
        &pool,
        owner,
        &recipe("Porridge", vec![MealType::Breakfast]),
        0,
    )
    .await
    .unwrap();
    create(&pool, owner, &recipe("Stew", vec![MealType::Dinner]), 0)
        .await
        .unwrap();
    create(&pool, owner, &recipe("Toast", Vec::new()), 0)
        .await
        .unwrap();
    let titles = |rows: Vec<crabbage_store::recipes::RecipeSummary>| {
        rows.into_iter().map(|row| row.title).collect::<Vec<_>>()
    };
    assert_eq!(
        titles(
            list(&pool, None, Some(MealType::Dinner), false, None, owner)
                .await
                .unwrap()
        ),
        ["Stew"]
    );
    assert_eq!(
        titles(list(&pool, None, None, false, None, owner).await.unwrap()),
        ["Porridge", "Stew", "Toast"]
    );
}
