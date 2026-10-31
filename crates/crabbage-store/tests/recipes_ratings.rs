//! Ratings: one per user per recipe that they can change, out-of-range stars are refused before the
//! database is touched, and the average and count cover every user.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::StoreError;
use crabbage_store::recipes::{RecipeInput, create, get, list, rate};
use crabbage_store::users::upsert_user;
use sqlx::sqlite::SqlitePool;

fn soup() -> RecipeInput {
    RecipeInput {
        title: "Soup".to_owned(),
        servings: 2,
        lines: vec![],
        steps: vec![],
        category_ids: vec![],
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn rating_rows(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM ratings")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn a_user_rates_once_and_changes_their_rating() {
    let (_dir, pool) = database("ratings-change").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    let id = create(&pool, owner, &soup(), 0).await.unwrap();

    assert!(rate(&pool, id, owner, 3).await.unwrap());
    assert!(rate(&pool, id, owner, 5).await.unwrap());

    let recipe = get(&pool, id, owner).await.unwrap().expect("exists");
    assert_eq!(recipe.my_stars, Some(5));
    assert_eq!(recipe.rating_count, 1);
    assert_eq!(rating_rows(&pool).await, 1);
}

#[tokio::test]
async fn stars_outside_one_to_five_are_refused_before_anything_is_stored() {
    let (_dir, pool) = database("ratings-range").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    let id = create(&pool, owner, &soup(), 0).await.unwrap();

    for stars in [0, 6, -1] {
        assert!(
            matches!(
                rate(&pool, id, owner, stars).await,
                Err(StoreError::InvalidStars)
            ),
            "{stars}"
        );
    }
    assert!(matches!(
        rate(&pool, 999, owner, 6).await,
        Err(StoreError::InvalidStars)
    ));
    assert_eq!(rating_rows(&pool).await, 0);
}

#[tokio::test]
async fn rating_a_missing_recipe_changes_nothing() {
    let (_dir, pool) = database("ratings-missing").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    assert!(!rate(&pool, 999, owner, 4).await.unwrap());
    assert_eq!(rating_rows(&pool).await, 0);
}

#[tokio::test]
async fn the_average_and_count_cover_every_user() {
    let (_dir, pool) = database("ratings-average").await;
    let owner = upsert_user(&pool, "owner", "Owner").await.unwrap();
    let friend = upsert_user(&pool, "friend", "Friend").await.unwrap();
    let guest = upsert_user(&pool, "guest", "Guest").await.unwrap();
    let id = create(&pool, owner, &soup(), 0).await.unwrap();
    let unrated = create(&pool, owner, &soup(), 0).await.unwrap();

    let before = get(&pool, id, guest).await.unwrap().expect("exists");
    assert_eq!((before.average, before.rating_count), (None, 0));

    rate(&pool, id, owner, 4).await.unwrap();
    rate(&pool, id, friend, 5).await.unwrap();

    let seen = get(&pool, id, guest).await.unwrap().expect("exists");
    assert_eq!(seen.average, Some(4.5));
    assert_eq!(seen.rating_count, 2);
    assert_eq!(seen.my_stars, None);
    assert_eq!(
        get(&pool, id, friend)
            .await
            .unwrap()
            .expect("exists")
            .my_stars,
        Some(5)
    );
    let summaries = list(&pool, None, false, guest).await.unwrap();
    let rated = summaries.iter().find(|row| row.id == id).expect("listed");
    assert_eq!((rated.average, rated.rating_count), (Some(4.5), 2));
    let empty = summaries
        .iter()
        .find(|row| row.id == unrated)
        .expect("listed");
    assert_eq!((empty.average, empty.rating_count), (None, 0));
}
