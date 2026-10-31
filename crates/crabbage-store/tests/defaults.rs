//! One-time catalog defaults for new and existing databases.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::catalog::{categories, ingredients, sections};
use crabbage_store::seed_defaults;
use std::collections::HashSet;

#[tokio::test]
async fn empty_catalog_is_seeded_once_and_deleted_defaults_stay_deleted() {
    let (_dir, pool) = database("defaults-empty").await;
    seed_defaults(&pool, "en").await.unwrap();
    seed_defaults(&pool, "nl").await.unwrap();
    let seeded_ingredients = ingredients::list(&pool).await.unwrap();
    let seeded_sections = sections::list(&pool).await.unwrap();
    let section_ids: HashSet<i64> = seeded_sections.iter().map(|section| section.id).collect();
    assert_eq!(seeded_ingredients.len(), 197);
    assert_eq!(categories::list(&pool).await.unwrap().len(), 12);
    assert_eq!(seeded_sections[0].name, "Produce");
    assert_eq!(seeded_sections[0].position, 0);
    assert!(
        seeded_ingredients
            .iter()
            .all(|ingredient| !ingredient.pending_review)
    );
    assert!(seeded_ingredients.iter().all(|ingredient| {
        ingredient
            .store_section_id
            .is_some_and(|id| section_ids.contains(&id))
    }));
    let onion = seeded_ingredients
        .iter()
        .find(|item| item.name == "Onion")
        .unwrap();
    assert!(onion.aliases.iter().any(|alias| alias == "onions"));
    ingredients::delete(&pool, onion.id).await.unwrap();
    seed_defaults(&pool, "nl").await.unwrap();
    assert_eq!(ingredients::list(&pool).await.unwrap().len(), 196);
    assert_eq!(sections::list(&pool).await.unwrap().len(), 12);
}

#[tokio::test]
async fn existing_catalog_is_preserved_and_marked_seeded() {
    let (_dir, pool) = database("defaults-existing").await;
    let section_id = sections::create(&pool, "Custom aisle").await.unwrap();
    let ingredient_id = ingredients::create(&pool, "Custom ingredient")
        .await
        .unwrap();
    ingredients::set_section(&pool, ingredient_id, Some(section_id))
        .await
        .unwrap();
    categories::create(&pool, "Custom category").await.unwrap();
    seed_defaults(&pool, "nl").await.unwrap();
    seed_defaults(&pool, "en").await.unwrap();
    assert_eq!(sections::list(&pool).await.unwrap().len(), 1);
    assert_eq!(
        ingredients::list(&pool).await.unwrap()[0].name,
        "Custom ingredient"
    );
    assert_eq!(
        categories::list(&pool).await.unwrap()[0].name,
        "Custom category"
    );
}
