//! The photo of a recipe: set, replaced and removed, and read back with the recipe and the list.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::recipes::{RecipeInput, create, get, list, set_image};
use crabbage_store::users::upsert_user;

fn soup() -> RecipeInput {
    RecipeInput {
        title: "Soup".to_owned(),
        servings: 2,
        category_ids: vec![],
        lines: vec![],
        steps: vec![],
    }
}

#[tokio::test]
async fn set_image_returns_the_replaced_name_and_reads_back() {
    let (_dir, pool) = database("recipes-images").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    let id = create(&pool, owner, &soup(), 0).await.unwrap();

    assert_eq!(set_image(&pool, id, Some("a.jpg")).await.unwrap(), None);
    assert_eq!(
        get(&pool, id, owner)
            .await
            .unwrap()
            .unwrap()
            .image
            .as_deref(),
        Some("a.jpg")
    );
    assert_eq!(
        list(&pool, None, false, None, owner).await.unwrap()[0]
            .image
            .as_deref(),
        Some("a.jpg")
    );

    assert_eq!(
        set_image(&pool, id, Some("b.png"))
            .await
            .unwrap()
            .as_deref(),
        Some("a.jpg")
    );
    assert_eq!(
        set_image(&pool, id, None).await.unwrap().as_deref(),
        Some("b.png")
    );
    assert_eq!(get(&pool, id, owner).await.unwrap().unwrap().image, None);
}

#[tokio::test]
async fn set_image_of_a_missing_recipe_changes_nothing() {
    let (_dir, pool) = database("recipes-images-missing").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    assert_eq!(set_image(&pool, 9, Some("a.jpg")).await.unwrap(), None);
    assert_eq!(get(&pool, 9, owner).await.unwrap(), None);
}
