//! The cook page: one step at a time with previous and next links, the mark-done toggle that keeps
//! the done steps in the address, the step dots, the swipe targets, and the link to it from the
//! recipe page.

use axum::http::StatusCode;
use axum::http::header::LOCATION;
use crabbage_store::recipes::{RecipeInput, create as create_recipe};

use crate::common::{Harness, body_text, header, sign_in_as};
use crate::recipes::create;

const OWNER: &str = "owner";

async fn page(harness: &Harness, token: &str, path: &str) -> String {
    let response = harness.get(path, Some(token)).await;
    assert_eq!(response.status(), StatusCode::OK, "{path}");
    body_text(response).await
}

/// A recipe with three steps as the recipe tests create it, and its owner's token.
async fn three_steps(harness: &Harness) -> (String, i64) {
    let owner = sign_in_as(harness, OWNER, &[]).await;
    let id = create(harness, &owner, "Pancakes").await;
    (owner, id)
}

#[tokio::test]
async fn the_first_step_shows_its_progress_text_and_no_previous() {
    let harness = Harness::start("cook-first").await;
    let (token, id) = three_steps(&harness).await;

    let body = page(&harness, &token, &format!("/recipes/{id}/cook")).await;
    assert!(body.contains("Step 1 of 3"), "{body}");
    assert!(body.contains("cook-title\">Pancakes</h1>"), "{body}");
    assert!(
        body.contains("class=\"cook-step\">Whisk the eggs"),
        "{body}"
    );
    assert!(body.contains("<p>2 min</p>"), "{body}");
    assert!(body.contains("value=\"1\" max=\"3\""), "{body}");
    assert!(!body.contains("Previous"), "{body}");
    assert!(!body.contains("Stir in the flour"), "{body}");
    assert!(
        body.contains(&format!("href=\"/recipes/{id}/cook?step=2\"")),
        "{body}"
    );
    assert!(body.contains("Next</a>"), "{body}");
    assert!(body.contains("aria-current=\"step\""), "{body}");
    assert_eq!(body.matches("aria-current=\"step\"").count(), 1, "{body}");
}

#[tokio::test]
async fn a_middle_step_links_to_its_neighbours() {
    let harness = Harness::start("cook-middle").await;
    let (token, id) = three_steps(&harness).await;

    let body = page(&harness, &token, &format!("/recipes/{id}/cook?step=2")).await;
    assert!(body.contains("Step 2 of 3"), "{body}");
    assert!(body.contains("Stir in the flour"), "{body}");
    assert!(
        body.contains(&format!("href=\"/recipes/{id}/cook?step=1\"")),
        "{body}"
    );
    assert!(
        body.contains(&format!("href=\"/recipes/{id}/cook?step=3\"")),
        "{body}"
    );
    assert!(body.contains("Previous</a>"), "{body}");
    assert!(body.contains("data-prev=\"/recipes/"), "{body}");
    assert!(body.contains("data-next=\"/recipes/"), "{body}");
}

#[tokio::test]
async fn the_last_step_says_done_cooking_and_goes_back_to_the_recipe() {
    let harness = Harness::start("cook-last").await;
    let (token, id) = three_steps(&harness).await;

    let body = page(&harness, &token, &format!("/recipes/{id}/cook?step=3")).await;
    assert!(body.contains("Step 3 of 3"), "{body}");
    assert!(body.contains("Fry until golden"), "{body}");
    assert!(body.contains("Done cooking</a>"), "{body}");
    assert!(body.contains(&format!("href=\"/recipes/{id}\"")), "{body}");
    assert!(!body.contains("step=4"), "{body}");
    assert!(!body.contains("Next</a>"), "{body}");
    assert!(!body.contains("data-next"), "{body}");
}

#[tokio::test]
async fn an_out_of_range_or_invalid_step_is_clamped() {
    let harness = Harness::start("cook-clamp").await;
    let (token, id) = three_steps(&harness).await;
    let path = format!("/recipes/{id}/cook");

    let body = page(&harness, &token, &format!("{path}?step=9")).await;
    assert!(body.contains("Step 3 of 3"), "{body}");
    assert!(body.contains("Fry until golden"), "{body}");

    for query in ["step=0", "step=abc", "step=-2", "step="] {
        let body = page(&harness, &token, &format!("{path}?{query}")).await;
        assert!(body.contains("Step 1 of 3"), "{query}: {body}");
        assert!(body.contains("Whisk the eggs"), "{query}: {body}");
    }
}

#[tokio::test]
async fn the_mark_done_toggle_adds_and_removes_the_current_step() {
    let harness = Harness::start("cook-toggle").await;
    let (token, id) = three_steps(&harness).await;
    let path = format!("/recipes/{id}/cook");

    let body = page(&harness, &token, &format!("{path}?step=2")).await;
    assert!(
        body.contains(&format!("href=\"{path}?step=2&amp;done=2\"")),
        "{body}"
    );
    assert!(body.contains("Mark done"), "{body}");
    assert!(body.contains("aria-pressed=\"false\""), "{body}");
    assert!(!body.contains("cook-done"), "{body}");

    let body = page(&harness, &token, &format!("{path}?step=2&done=2")).await;
    assert!(body.contains(&format!("href=\"{path}?step=2\"")), "{body}");
    assert!(body.contains("Not done"), "{body}");
    assert!(body.contains("aria-pressed=\"true\""), "{body}");
    assert!(body.contains("class=\"cook-done\">✓ Done</p>"), "{body}");
}

#[tokio::test]
async fn done_steps_stay_marked_while_navigating() {
    let harness = Harness::start("cook-stay-done").await;
    let (token, id) = three_steps(&harness).await;
    let path = format!("/recipes/{id}/cook");

    let body = page(&harness, &token, &format!("{path}?step=1&done=1")).await;
    assert!(
        body.contains(&format!("href=\"{path}?step=2&amp;done=1\"")),
        "{body}"
    );
    assert!(body.contains("class=\"cook-dot done\""), "{body}");
    assert_eq!(body.matches("class=\"cook-dot done\"").count(), 1, "{body}");

    let body = page(&harness, &token, &format!("{path}?step=3&done=1")).await;
    assert!(body.contains("Step 3 of 3"), "{body}");
    assert!(body.contains("class=\"cook-dot done\""), "{body}");
    assert!(!body.contains("cook-done\">"), "{body}");
}

#[tokio::test]
async fn done_steps_are_deduplicated_sorted_and_invalid_entries_ignored() {
    let harness = Harness::start("cook-dedupe").await;
    let (token, id) = three_steps(&harness).await;
    let path = format!("/recipes/{id}/cook");

    let body = page(
        &harness,
        &token,
        &format!("{path}?step=1&done=3,x,2,3,,0,9,-1"),
    )
    .await;
    assert_eq!(body.matches("class=\"cook-dot done\"").count(), 2, "{body}");
    assert!(body.contains("step=2&amp;done=2,3\""), "{body}");
    assert!(body.contains("done=1,2,3"), "{body}");
}

#[tokio::test]
async fn the_servings_are_kept_in_every_link() {
    let harness = Harness::start("cook-servings").await;
    let (token, id) = three_steps(&harness).await;
    let path = format!("/recipes/{id}/cook");

    let body = page(&harness, &token, &format!("{path}?step=2&servings=8")).await;
    assert!(body.contains("step=1&amp;servings=8"), "{body}");
    assert!(body.contains("step=3&amp;servings=8"), "{body}");

    let body = page(&harness, &token, &format!("{path}?step=3&servings=8")).await;
    assert!(
        body.contains(&format!("href=\"/recipes/{id}?servings=8\"")),
        "{body}"
    );

    let body = page(&harness, &token, &format!("{path}?step=3&servings=1000")).await;
    assert!(
        body.contains(&format!("href=\"/recipes/{id}?servings=4\"")),
        "{body}"
    );
}

#[tokio::test]
async fn the_swipe_container_carries_the_previous_and_next_addresses() {
    let harness = Harness::start("cook-swipe").await;
    let (token, id) = three_steps(&harness).await;
    let path = format!("/recipes/{id}/cook");

    let body = page(&harness, &token, &format!("{path}?step=2")).await;
    assert!(body.contains("class=\"cook-card\""), "{body}");
    assert!(
        body.contains(&format!("data-prev=\"{path}?step=1\"")),
        "{body}"
    );
    assert!(
        body.contains(&format!("data-next=\"{path}?step=3\"")),
        "{body}"
    );

    let body = page(&harness, &token, &format!("{path}?step=1")).await;
    assert!(!body.contains("data-prev"), "{body}");
}

#[tokio::test]
async fn a_recipe_without_steps_shows_a_message_in_place_of_the_step() {
    let harness = Harness::start("cook-no-steps").await;
    let token = sign_in_as(&harness, OWNER, &[]).await;
    let owner: i64 = sqlx::query_scalar("SELECT id FROM users WHERE subject = ?")
        .bind(OWNER)
        .fetch_one(&harness.pool)
        .await
        .expect("the user exists");
    let id = create_recipe(
        &harness.pool,
        owner,
        &RecipeInput {
            title: "Water".to_owned(),
            servings: 1,
            lines: Vec::new(),
            steps: Vec::new(),
            category_ids: Vec::new(),
        },
        0,
    )
    .await
    .expect("recipe is stored");

    let body = page(&harness, &token, &format!("/recipes/{id}/cook")).await;
    assert!(body.contains("This recipe has no steps yet."), "{body}");
    assert!(body.contains(&format!("href=\"/recipes/{id}\"")), "{body}");
    assert!(!body.contains("cook-step"), "{body}");
    assert!(!body.contains("Step 1 of"), "{body}");
}

#[tokio::test]
async fn an_unknown_or_invalid_recipe_says_it_does_not_exist() {
    let harness = Harness::start("cook-missing").await;
    let token = sign_in_as(&harness, OWNER, &[]).await;
    for path in ["/recipes/999/cook", "/recipes/abc/cook"] {
        let body = page(&harness, &token, path).await;
        assert!(body.contains("does not exist"), "{path}: {body}");
    }
}

#[tokio::test]
async fn signed_out_visitors_are_sent_to_sign_in() {
    let harness = Harness::start("cook-signed-out").await;
    let (_, id) = three_steps(&harness).await;
    let response = harness.get(&format!("/recipes/{id}/cook"), None).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(header(&response, LOCATION).as_deref(), Some("/auth/login"));
}

#[tokio::test]
async fn the_recipe_page_links_to_cooking_and_keeps_the_servings() {
    let harness = Harness::start("cook-start").await;
    let (token, id) = three_steps(&harness).await;

    let body = page(&harness, &token, &format!("/recipes/{id}")).await;
    assert!(body.contains("Start cooking</a>"), "{body}");
    assert!(
        body.contains(&format!("href=\"/recipes/{id}/cook\"")),
        "{body}"
    );

    let body = page(&harness, &token, &format!("/recipes/{id}?servings=8")).await;
    assert!(
        body.contains(&format!("href=\"/recipes/{id}/cook?servings=8\"")),
        "{body}"
    );
}
