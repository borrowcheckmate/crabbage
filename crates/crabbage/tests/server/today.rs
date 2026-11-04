use axum::http::StatusCode;
use crabbage_store::planner::{PlanEntry, set_entry};
use crabbage_store::shopping::add_text;

use crate::common::{Harness, body_text, sign_in_as};
use crate::recipes::create;

const HOME: &str = "crabbage-household-today-home";

fn current_date() -> (i64, String) {
    let day = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock is after the epoch")
            .as_secs()
            / 86_400,
    )
    .expect("day fits in the date range");
    (
        day,
        crabbage_domain::dates::format_iso_date(day).expect("date is valid"),
    )
}

async fn household(harness: &Harness) -> i64 {
    sqlx::query_scalar(
        "SELECT uh.household_id FROM user_households uh JOIN users u ON u.id = uh.user_id WHERE u.subject = 'today-member'",
    )
    .fetch_one(&harness.pool)
    .await
    .expect("household exists")
}

/// Today's page for a household with a recipe tonight, a rest day tomorrow, the same recipe the day
/// after and one item to buy, with the recipe's id.
async fn planned_today(name: &str) -> (String, i64) {
    let harness = Harness::start(name).await;
    let token = sign_in_as(&harness, "today-member", &[HOME]).await;
    let home = household(&harness).await;
    let recipe = create(&harness, &token, "Planned dinner").await;
    let (day, today) = current_date();
    for entry in [
        PlanEntry {
            date: today,
            recipe_id: Some(recipe),
            servings: 3,
        },
        PlanEntry {
            date: crabbage_domain::dates::format_iso_date(day + 1).expect("date"),
            recipe_id: None,
            servings: 4,
        },
        PlanEntry {
            date: crabbage_domain::dates::format_iso_date(day + 2).expect("date"),
            recipe_id: Some(recipe),
            servings: 5,
        },
    ] {
        set_entry(&harness.pool, home, &entry)
            .await
            .expect("plan entry saved");
    }
    add_text(&harness.pool, home, "Olive oil")
        .await
        .expect("shopping item saved");
    let response = harness.get("/", Some(&token)).await;
    assert_eq!(response.status(), StatusCode::OK);
    (body_text(response).await, recipe)
}

#[tokio::test]
async fn today_shows_household_meal_upcoming_preview_and_shopping_count() {
    let (body, recipe) = planned_today("today-planned").await;
    for expected in [
        "today-home",
        "Tonight’s plan",
        "Planned dinner",
        "Start cooking",
        "Rest day",
        "3 servings",
        "5 servings",
        "1 item to pick up",
        "/shopping",
    ] {
        assert!(body.contains(expected), "{expected}: {body}");
    }
    assert!(
        body.contains(&format!("/recipes/{recipe}/cook?servings=3")),
        "{body}"
    );
}

#[tokio::test]
async fn today_opens_planned_recipes_at_their_servings() {
    let (body, recipe) = planned_today("today-links").await;
    assert!(
        body.contains(&format!(
            "<h2><a href=\"/recipes/{recipe}?servings=3\">Planned dinner</a></h2>"
        )),
        "today's recipe opens from its title: {body}"
    );
    assert!(
        body.contains(&format!(
            "<a href=\"/recipes/{recipe}?servings=3\" tabindex=\"-1\" aria-hidden=\"true\" aria-label=\"Planned dinner\" class=\"today-feature-photo\">"
        )),
        "today's recipe opens from its labelled photo, skipped by keyboard and screen readers: {body}"
    );
    assert!(
        body.contains(&format!(
            "<a href=\"/recipes/{recipe}?servings=5\" class=\"today-next\">"
        )),
        "an upcoming recipe opens from its row: {body}"
    );
    assert_eq!(
        body.matches("class=\"today-next\">").count(),
        2,
        "one upcoming recipe row and one rest day: {body}"
    );
    assert_eq!(
        body.matches("\" class=\"today-next\">").count(),
        1,
        "only the recipe row is a link: {body}"
    );
    assert!(
        body.contains("<div class=\"today-next\"><span>"),
        "a rest day is not a link: {body}"
    );
}

#[tokio::test]
async fn today_has_empty_and_no_household_states() {
    let harness = Harness::start("today-empty").await;
    let token = sign_in_as(&harness, "today-empty-member", &[HOME]).await;
    let body = body_text(harness.get("/", Some(&token)).await).await;
    assert!(body.contains("Nothing is planned for today."), "{body}");
    assert!(
        body.contains("No meals are planned for the next few days."),
        "{body}"
    );
    let home: i64 = sqlx::query_scalar(
        "SELECT uh.household_id FROM user_households uh JOIN users u ON u.id = uh.user_id WHERE u.subject = 'today-empty-member'",
    )
    .fetch_one(&harness.pool)
    .await
    .expect("household exists");
    let (_, date) = current_date();
    set_entry(
        &harness.pool,
        home,
        &PlanEntry {
            date,
            recipe_id: None,
            servings: 4,
        },
    )
    .await
    .expect("rest day saved");
    let body = body_text(harness.get("/", Some(&token)).await).await;
    assert!(body.contains("Rest day"), "{body}");
    assert!(!body.contains("Start cooking"), "{body}");

    let no_home = sign_in_as(&harness, "today-no-household", &[]).await;
    let body = body_text(harness.get("/", Some(&no_home)).await).await;
    assert!(
        body.contains("This account has no household. Ask an administrator to add one."),
        "{body}"
    );
    assert!(!body.contains("items to pick up"), "{body}");
}

#[tokio::test]
async fn changing_households_changes_today_data_without_leaking_the_previous_plan() {
    let harness = Harness::start("today-household-switch").await;
    let token = sign_in_as(
        &harness,
        "today-switch-member",
        &[HOME, "crabbage-household-today-away"],
    )
    .await;
    let home: i64 = sqlx::query_scalar("SELECT id FROM households WHERE group_name = ?")
        .bind(HOME)
        .fetch_one(&harness.pool)
        .await
        .expect("home household exists");
    let away: i64 = sqlx::query_scalar(
        "SELECT id FROM households WHERE group_name = 'crabbage-household-today-away'",
    )
    .fetch_one(&harness.pool)
    .await
    .expect("away household exists");
    let home_recipe = create(&harness, &token, "Home dinner").await;
    let away_recipe = create(&harness, &token, "Away dinner").await;
    let (_, date) = current_date();
    for (household, recipe) in [(home, home_recipe), (away, away_recipe)] {
        set_entry(
            &harness.pool,
            household,
            &PlanEntry {
                date: date.clone(),
                recipe_id: Some(recipe),
                servings: 2,
            },
        )
        .await
        .expect("plan entry saved");
    }
    let switched = harness
        .post_form("/household", Some(&token), &format!("household_id={home}"))
        .await;
    assert_eq!(switched.status(), StatusCode::SEE_OTHER);
    let body = body_text(harness.get("/", Some(&token)).await).await;
    assert!(body.contains("Home dinner"), "{body}");
    assert!(!body.contains("Away dinner"), "{body}");
}

#[tokio::test]
async fn plan_week_navigation_and_selected_day_editor_keep_dates_and_actions_visible() {
    let harness = Harness::start("plan-layout").await;
    let token = sign_in_as(&harness, "plan-layout-member", &[HOME]).await;
    let body = body_text(
        harness
            .get("/plan?week=2024-02-26&day=2024-02-28", Some(&token))
            .await,
    )
    .await;
    for expected in [
        "Weekly plan",
        "Previous week",
        "Next week",
        "Planning preferences",
        "name=\"recipe\"",
        "name=\"servings\"",
        "Reroll",
        "No recipe selected",
        "aria-current=\"date\"",
    ] {
        assert!(body.contains(expected), "{expected}: {body}");
    }
    assert!(
        body.contains("/plan?week=2024-02-26&amp;day=2024-02-28"),
        "{body}"
    );
}
