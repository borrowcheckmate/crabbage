use axum::http::StatusCode;
use crabbage_store::planner::{PlanEntry, PlannerSettings, entries, save_settings, settings};

use crate::common::{Harness, body_text, sign_in_as};
use crate::recipes::create;

const HOME: &str = "crabbage-household-planner-home";
const AWAY: &str = "crabbage-household-planner-away";
const WEEK: &str = "2024-02-26";
const CATEGORY: i64 = 71;

async fn setup(name: &str) -> (Harness, String, i64, Vec<i64>) {
    let harness = Harness::start(name).await;
    let token = sign_in_as(&harness, "planner-member", &[HOME]).await;
    let owner: i64 = sqlx::query_scalar("SELECT id FROM users WHERE subject = 'planner-member'")
        .fetch_one(&harness.pool)
        .await
        .expect("member exists");
    sqlx::query("INSERT INTO categories (id, name) VALUES (?, 'Planner category')")
        .bind(CATEGORY)
        .execute(&harness.pool)
        .await
        .expect("category created");
    let first = create(&harness, &token, "Planner recipe A").await;
    let second = create(&harness, &token, "Planner recipe B").await;
    for recipe in [first, second] {
        sqlx::query("INSERT INTO recipe_categories (recipe_id, category_id) VALUES (?, ?)")
            .bind(recipe)
            .bind(CATEGORY)
            .execute(&harness.pool)
            .await
            .expect("category assigned");
    }
    save_settings(
        &harness.pool,
        household(&harness, owner).await,
        &PlannerSettings {
            lookback_weeks: 0,
            servings: 4,
            weekdays: [
                Some(CATEGORY),
                Some(CATEGORY),
                Some(CATEGORY),
                Some(CATEGORY),
                Some(CATEGORY),
                Some(CATEGORY),
                Some(CATEGORY),
            ],
        },
    )
    .await
    .expect("settings saved");
    (harness, token, owner, vec![first, second])
}

async fn household(harness: &Harness, user: i64) -> i64 {
    sqlx::query_scalar("SELECT household_id FROM user_households WHERE user_id = ?")
        .bind(user)
        .fetch_one(&harness.pool)
        .await
        .expect("household exists")
}

async fn post(harness: &Harness, token: &str, path: &str, form: &str) -> StatusCode {
    harness.post_form(path, Some(token), form).await.status()
}

#[tokio::test]
async fn generate_keeps_existing_entries_and_reroll_changes_only_the_requested_day() {
    let (harness, token, owner, recipes) = setup("planner-generate").await;
    let home = household(&harness, owner).await;
    crabbage_store::planner::set_entry(
        &harness.pool,
        home,
        &PlanEntry {
            date: WEEK.to_owned(),
            recipe_id: Some(recipes[0]),
            servings: 6,
        },
    )
    .await
    .expect("entry saved");
    assert_eq!(
        post(&harness, &token, "/plan/generate", "week=2024-02-26").await,
        StatusCode::SEE_OTHER
    );
    let before = entries(&harness.pool, home, WEEK, "2024-03-03")
        .await
        .expect("week read");
    assert_eq!(before.len(), 7);
    assert_eq!(before[0].recipe_id, Some(recipes[0]));
    assert_eq!(before[0].servings, 6);
    assert!(
        before
            .iter()
            .all(|entry| entry.recipe_id.is_some_and(|id| recipes.contains(&id)))
    );
    assert_eq!(
        post(&harness, &token, "/plan/2024-02-27/reroll", "").await,
        StatusCode::SEE_OTHER
    );
    let after = entries(&harness.pool, home, WEEK, "2024-03-03")
        .await
        .expect("week read");
    assert_eq!(after[0], before[0]);
    assert_ne!(after[1].recipe_id, before[1].recipe_id);
    assert_eq!(after[2..], before[2..]);
}

#[tokio::test]
async fn manual_recipe_rest_servings_settings_and_household_scope_work() {
    let (harness, token, owner, recipes) = setup("planner-manual").await;
    let home = household(&harness, owner).await;
    assert_eq!(
        post(
            &harness,
            &token,
            "/plan/2024-02-26/recipe",
            &format!("recipe={}", recipes[1])
        )
        .await,
        StatusCode::SEE_OTHER
    );
    assert_eq!(
        post(&harness, &token, "/plan/2024-02-26/servings", "servings=8").await,
        StatusCode::SEE_OTHER
    );
    assert_eq!(
        post(&harness, &token, "/plan/2024-02-27/recipe", "recipe=rest").await,
        StatusCode::SEE_OTHER
    );
    let rows = entries(&harness.pool, home, WEEK, "2024-02-27")
        .await
        .expect("entries read");
    assert_eq!(rows[0].recipe_id, Some(recipes[1]));
    assert_eq!(rows[0].servings, 8);
    assert_eq!(rows[1].recipe_id, None);
    settings_round_trip(&harness, &token, home).await;
    let away = sign_in_as(&harness, "planner-away", &[AWAY]).await;
    let response = harness.get("/plan?week=2024-02-26", Some(&away)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let page = body_text(response).await;
    assert!(
        !page.contains(&format!("href=\"/recipes/{}?servings=", recipes[1])),
        "{page}"
    );
}

async fn settings_round_trip(harness: &Harness, token: &str, home: i64) {
    let settings_form = "lookback_weeks=3&servings=5&monday=71&tuesday=rest&wednesday=71&thursday=rest&friday=71&saturday=rest&sunday=71";
    assert_eq!(
        post(harness, token, "/plan/settings", settings_form).await,
        StatusCode::SEE_OTHER
    );
    let saved = settings(&harness.pool, home).await.expect("settings read");
    assert_eq!(saved.lookback_weeks, 3);
    assert_eq!(saved.servings, 5);
    assert_eq!(
        saved.weekdays,
        [Some(71), None, Some(71), None, Some(71), None, Some(71)]
    );
    assert_eq!(
        post(harness, token, "/plan/generate", "week=2024-03-04").await,
        StatusCode::SEE_OTHER
    );
    let generated = entries(&harness.pool, home, "2024-03-04", "2024-03-10")
        .await
        .expect("generated week read");
    assert_eq!(
        generated
            .iter()
            .find(|entry| entry.date == "2024-03-05")
            .and_then(|entry| entry.recipe_id),
        None
    );
    let planned = body_text(harness.get("/plan?week=2024-03-04", Some(token)).await).await;
    assert!(planned.contains("<strong>Rest day</strong>"), "{planned}");
}

#[tokio::test]
async fn planner_forms_reject_invalid_values_and_week_view_links_with_servings() {
    let (harness, token, owner, recipes) = setup("planner-validation").await;
    let home = household(&harness, owner).await;
    crabbage_store::planner::set_entry(
        &harness.pool,
        home,
        &PlanEntry {
            date: WEEK.to_owned(),
            recipe_id: Some(recipes[0]),
            servings: 9,
        },
    )
    .await
    .expect("entry saved");
    assert_eq!(
        post(
            &harness,
            &token,
            "/plan/2024-02-26/servings",
            "servings=100"
        )
        .await,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        post(
            &harness,
            &token,
            "/plan/2024-02-26/recipe",
            "recipe=invalid"
        )
        .await,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(post(&harness, &token, "/plan/settings", "lookback_weeks=-1&servings=4&monday=71&tuesday=71&wednesday=71&thursday=71&friday=71&saturday=71&sunday=71").await, StatusCode::BAD_REQUEST);
    let response = harness.get("/plan?week=2024-02-26", Some(&token)).await;
    let page = body_text(response).await;
    assert!(
        page.contains(&format!("/recipes/{}?servings=9", recipes[0])),
        "{page}"
    );
    assert!(
        page.contains(&format!("/recipes/{}/cook?servings=9", recipes[0])),
        "{page}"
    );
}

#[tokio::test]
async fn plan_page_adds_the_week_to_shopping_once() {
    let (harness, token, owner, recipes) = setup("planner-shopping").await;
    let home = household(&harness, owner).await;
    crabbage_store::planner::set_entry(
        &harness.pool,
        home,
        &PlanEntry {
            date: WEEK.to_owned(),
            recipe_id: Some(recipes[0]),
            servings: 8,
        },
    )
    .await
    .expect("entry saved");
    let page = body_text(harness.get("/plan?week=2024-02-26", Some(&token)).await).await;
    assert!(page.contains("Add week to shopping list"), "{page}");
    assert!(
        page.contains(r#"aria-label="Increase servings">+</button>"#),
        "{page}"
    );
    assert_eq!(
        post(&harness, &token, "/plan/shopping", "week=2024-02-26").await,
        StatusCode::SEE_OTHER
    );
    assert_eq!(
        post(&harness, &token, "/plan/shopping", "week=2024-02-26").await,
        StatusCode::SEE_OTHER
    );
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM shopping_items WHERE household_id = ? AND plan_entry_id IS NOT NULL",
    )
    .bind(home)
    .fetch_one(&harness.pool)
    .await
    .expect("shopping count");
    assert_eq!(count, 3);
}
