//! Household planner settings and dated entries stay isolated and validate their inputs.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::StoreError;
use crabbage_store::planner::{
    PlanEntry, PlannerSettings, categories_exist, entries, history, recipe_exists,
    recipes_in_categories, remove_entry, save_settings, set_entry, settings,
};
use crabbage_store::users::upsert_user;
use sqlx::sqlite::SqlitePool;

#[expect(
    clippy::unwrap_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn household(pool: &SqlitePool, group: &str) -> i64 {
    sqlx::query_scalar("INSERT INTO households (group_name) VALUES (?) RETURNING id")
        .bind(group)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[expect(
    clippy::unwrap_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn planned_entries(pool: &SqlitePool, household: i64) -> Vec<PlanEntry> {
    entries(pool, household, "2026-10-31", "2026-11-01")
        .await
        .unwrap()
}

#[expect(
    clippy::unwrap_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn recipe(pool: &SqlitePool) -> i64 {
    let owner = upsert_user(pool, "planner-owner", "Planner").await.unwrap();
    sqlx::query_scalar(
        "INSERT INTO recipes (owner_id, title, servings, created_at) \
         VALUES (?, 'Soup', 4, 0) RETURNING id",
    )
    .bind(owner)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn settings_default_then_round_trip_per_household() {
    let (_dir, pool) = database("planner-settings").await;
    let home = household(&pool, "crabbage-household-home").await;
    let other = household(&pool, "crabbage-household-other").await;
    assert_eq!(
        settings(&pool, home).await.unwrap(),
        PlannerSettings {
            lookback_weeks: 1,
            servings: 4,
            weekdays: [None; 7],
        }
    );
    let saved = PlannerSettings {
        lookback_weeks: 3,
        servings: 6,
        weekdays: [Some(1), None, Some(2), None, Some(1), None, None],
    };
    sqlx::query("INSERT INTO categories (id, name) VALUES (1, 'Breakfast'), (2, 'Dinner')")
        .execute(&pool)
        .await
        .unwrap();
    save_settings(&pool, home, &saved).await.unwrap();
    assert_eq!(settings(&pool, home).await.unwrap(), saved);
    assert_eq!(settings(&pool, other).await.unwrap().lookback_weeks, 1);
}

#[tokio::test]
async fn entries_upsert_recipe_and_rest_and_filter_households_and_dates() {
    let (_dir, pool) = database("planner-entries").await;
    let home = household(&pool, "crabbage-household-home").await;
    let other = household(&pool, "crabbage-household-other").await;
    let recipe = recipe(&pool).await;
    let first = PlanEntry {
        date: "2026-10-31".to_owned(),
        recipe_id: Some(recipe),
        servings: 2,
    };
    set_entry(&pool, home, &first).await.unwrap();
    set_entry(
        &pool,
        home,
        &PlanEntry {
            date: first.date.clone(),
            recipe_id: None,
            servings: 5,
        },
    )
    .await
    .unwrap();
    set_entry(
        &pool,
        home,
        &PlanEntry {
            date: "2026-11-01".to_owned(),
            recipe_id: Some(recipe),
            servings: 4,
        },
    )
    .await
    .unwrap();
    assert_eq!(
        planned_entries(&pool, home).await,
        [
            PlanEntry {
                date: "2026-10-31".to_owned(),
                recipe_id: None,
                servings: 5,
            },
            PlanEntry {
                date: "2026-11-01".to_owned(),
                recipe_id: Some(recipe),
                servings: 4,
            },
        ]
    );
    assert_eq!(planned_entries(&pool, other).await, Vec::<PlanEntry>::new());
    assert_eq!(
        history(&pool, home, "2026-11-01", "2026-10-31")
            .await
            .unwrap(),
        [(recipe, "2026-11-01".to_owned())]
    );
    assert!(remove_entry(&pool, home, "2026-10-31").await.unwrap());
    assert!(!remove_entry(&pool, other, "2026-11-01").await.unwrap());
}

#[tokio::test]
async fn invalid_settings_servings_and_dates_are_rejected() {
    let (_dir, pool) = database("planner-invalid").await;
    let home = household(&pool, "crabbage-household-home").await;
    assert!(
        sqlx::query(
            "INSERT INTO planner_weekdays (household_id, weekday, category_id) VALUES (?, 7, NULL)",
        )
        .bind(home)
        .execute(&pool)
        .await
        .is_err()
    );
    for invalid in [
        PlannerSettings {
            lookback_weeks: -1,
            servings: 4,
            weekdays: [None; 7],
        },
        PlannerSettings {
            lookback_weeks: 1,
            servings: 0,
            weekdays: [None; 7],
        },
    ] {
        assert!(matches!(
            save_settings(&pool, home, &invalid).await,
            Err(StoreError::InvalidPlannerSettings)
        ));
    }
    assert!(matches!(
        set_entry(
            &pool,
            home,
            &PlanEntry {
                date: "2026-02-29".to_owned(),
                recipe_id: None,
                servings: 4,
            }
        )
        .await,
        Err(StoreError::InvalidDate)
    ));
    assert!(matches!(
        set_entry(
            &pool,
            home,
            &PlanEntry {
                date: "2026-03-01".to_owned(),
                recipe_id: None,
                servings: 0,
            }
        )
        .await,
        Err(StoreError::InvalidPlannerSettings)
    ));
    assert!(matches!(
        entries(&pool, home, "2026-02-30", "2026-03-01").await,
        Err(StoreError::InvalidDate)
    ));
}

#[tokio::test]
async fn planner_recipe_lookups_cover_candidates_and_existence() {
    let (_dir, pool) = database("planner-lookups").await;
    let first = recipe(&pool).await;
    let second = recipe(&pool).await;
    sqlx::query("INSERT INTO categories (id, name) VALUES (1, 'Breakfast'), (2, 'Dinner')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO recipe_categories (recipe_id, category_id) VALUES (?, 2), (?, 1)")
        .bind(second)
        .bind(first)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        recipes_in_categories(&pool, &[2, 1]).await.unwrap(),
        [(1, first), (2, second)]
    );
    assert_eq!(recipes_in_categories(&pool, &[]).await.unwrap(), []);
    assert!(recipe_exists(&pool, first).await.unwrap());
    assert!(!recipe_exists(&pool, i64::MAX).await.unwrap());
    assert!(
        categories_exist(&pool, &[Some(1), Some(2), None, None, None, None, None])
            .await
            .unwrap()
    );
    assert!(
        !categories_exist(
            &pool,
            &[Some(1), Some(i64::MAX), None, None, None, None, None]
        )
        .await
        .unwrap()
    );
    assert!(categories_exist(&pool, &[None; 7]).await.unwrap());
}
