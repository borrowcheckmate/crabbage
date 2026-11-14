//! Ingredient allergens: replaced as a set in a fixed order, merged along with the ingredient,
//! and seeded once for ingredients named like the seeded catalogue in either language.

use crate::common::database;
use crabbage_domain::allergens::Allergen;
use crabbage_store::catalog::allergens::set_allergens;
use crabbage_store::catalog::ingredients::{self, create, merge};
use crabbage_store::seed_defaults;
use sqlx::sqlite::SqlitePool;

#[expect(
    clippy::unwrap_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn allergens_of(pool: &SqlitePool, name: &str) -> Vec<Allergen> {
    ingredients::list(pool)
        .await
        .unwrap()
        .into_iter()
        .find(|ingredient| ingredient.name == name)
        .map(|ingredient| ingredient.allergens)
        .unwrap_or_default()
}

#[tokio::test]
async fn allergens_are_replaced_in_order_and_a_missing_ingredient_is_false() {
    let (_dir, pool) = database("allergens-set").await;
    let pesto = create(&pool, "Pesto").await.unwrap();
    assert!(
        set_allergens(
            &pool,
            pesto,
            &[Allergen::TreeNuts, Allergen::Milk, Allergen::Milk]
        )
        .await
        .unwrap()
    );
    assert_eq!(
        allergens_of(&pool, "Pesto").await,
        [Allergen::Milk, Allergen::TreeNuts]
    );
    assert!(set_allergens(&pool, pesto, &[]).await.unwrap());
    assert_eq!(allergens_of(&pool, "Pesto").await, Vec::<Allergen>::new());
    assert!(
        !set_allergens(&pool, i64::MAX, &[Allergen::Milk])
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn a_merged_ingredient_brings_its_allergens() {
    let (_dir, pool) = database("allergens-merge").await;
    let mayo = create(&pool, "Mayonnaise").await.unwrap();
    let aioli = create(&pool, "Aioli").await.unwrap();
    set_allergens(&pool, mayo, &[Allergen::Eggs]).await.unwrap();
    set_allergens(&pool, aioli, &[Allergen::Eggs, Allergen::Mustard])
        .await
        .unwrap();
    assert!(merge(&pool, aioli, mayo).await.unwrap());
    assert_eq!(
        allergens_of(&pool, "Mayonnaise").await,
        [Allergen::Eggs, Allergen::Mustard]
    );
}

#[tokio::test]
async fn a_new_catalogue_is_seeded_with_its_allergens() {
    let (_dir, pool) = database("allergens-seed-new").await;
    seed_defaults(&pool, "en").await.unwrap();
    assert_eq!(allergens_of(&pool, "Milk").await, [Allergen::Milk]);
    assert_eq!(
        allergens_of(&pool, "Soy sauce").await,
        [Allergen::Gluten, Allergen::Soy]
    );
    assert_eq!(
        allergens_of(&pool, "Chicken breast").await,
        [Allergen::Meat]
    );
    assert_eq!(allergens_of(&pool, "Onion").await, Vec::<Allergen>::new());
}

#[tokio::test]
async fn an_existing_catalogue_gets_seeded_allergens_once_by_either_name() {
    let (_dir, pool) = database("allergens-seed-existing").await;
    sqlx::query("INSERT INTO catalog_seed (id) VALUES (1)")
        .execute(&pool)
        .await
        .unwrap();
    let flour = create(&pool, "Bloem").await.unwrap();
    create(&pool, "egg").await.unwrap();
    create(&pool, "Dragonfruit").await.unwrap();
    seed_defaults(&pool, "en").await.unwrap();
    assert_eq!(allergens_of(&pool, "Bloem").await, [Allergen::Gluten]);
    assert_eq!(allergens_of(&pool, "Egg").await, [Allergen::Eggs]);
    assert_eq!(
        allergens_of(&pool, "Dragonfruit").await,
        Vec::<Allergen>::new()
    );
    assert_eq!(ingredients::list(&pool).await.unwrap().len(), 3);

    set_allergens(&pool, flour, &[]).await.unwrap();
    seed_defaults(&pool, "en").await.unwrap();
    assert_eq!(allergens_of(&pool, "Bloem").await, Vec::<Allergen>::new());
}
