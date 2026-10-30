//! Owners add and remove editors. Adding twice changes nothing, and a missing recipe or user is
//! refused.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::recipes::{RecipeInput, add_editor, create, get, list_users, remove_editor};
use crabbage_store::users::upsert_user;

fn soup() -> RecipeInput {
    RecipeInput {
        title: "Soup".to_owned(),
        servings: 2,
        lines: vec![],
        steps: vec![],
    }
}

#[tokio::test]
async fn editors_are_added_listed_and_removed() {
    let (_dir, pool) = database("recipes-editors").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    let friend = upsert_user(&pool, "friend", "Friend").await.unwrap();
    let id = create(&pool, owner, &soup(), 0).await.unwrap();

    assert!(add_editor(&pool, id, friend).await.unwrap());
    assert!(add_editor(&pool, id, friend).await.unwrap());
    let editors: Vec<String> = get(&pool, id)
        .await
        .unwrap()
        .expect("the recipe exists")
        .editors
        .into_iter()
        .map(|person| person.name)
        .collect();
    assert_eq!(editors, ["Friend"]);

    assert!(remove_editor(&pool, id, friend).await.unwrap());
    assert!(!remove_editor(&pool, id, friend).await.unwrap());
    assert_eq!(
        get(&pool, id).await.unwrap().expect("exists").editors,
        Vec::new()
    );
}

#[tokio::test]
async fn a_missing_recipe_or_user_is_not_an_editor() {
    let (_dir, pool) = database("recipes-editors-missing").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    let id = create(&pool, owner, &soup(), 0).await.unwrap();
    assert!(!add_editor(&pool, 99, owner).await.unwrap());
    assert!(!add_editor(&pool, id, 99).await.unwrap());
}

#[tokio::test]
async fn users_are_listed_by_name() {
    let (_dir, pool) = database("recipes-users").await;
    upsert_user(&pool, "b", "Bram").await.unwrap();
    upsert_user(&pool, "a", "Ada").await.unwrap();
    let names: Vec<String> = list_users(&pool)
        .await
        .unwrap()
        .into_iter()
        .map(|person| person.name)
        .collect();
    assert_eq!(names, ["Ada", "Bram"]);
}
