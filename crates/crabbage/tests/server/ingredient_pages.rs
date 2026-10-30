//! The ingredient list and its search, and one ingredient's page. Delete and merge show their
//! question only once the link for it has been followed.

use axum::http::StatusCode;

use crabbage_store::catalog::{ingredients, sections};

use crate::common::{Harness, body_text, session_token};

async fn admin_token(harness: &Harness) -> String {
    harness.issuer.set_groups(&["crabbage-admin"]);
    session_token(&harness.sign_in().await).expect("sign-in sets a session")
}

/// A store with Tomato (in Produce, alias Pomodoro), Onion and the pending Okra. Returns the ids of
/// Tomato and Onion.
async fn seed(harness: &Harness) -> (i64, i64) {
    let produce = sections::create(&harness.pool, "Produce")
        .await
        .expect("the seed row is written");
    let tomato = ingredients::create(&harness.pool, "Tomato")
        .await
        .expect("the seed row is written");
    let onion = ingredients::create(&harness.pool, "Onion")
        .await
        .expect("the seed row is written");
    ingredients::set_section(&harness.pool, tomato, Some(produce))
        .await
        .expect("the seed row is written");
    ingredients::add_alias(&harness.pool, tomato, "Pomodoro")
        .await
        .expect("the seed row is written");
    sqlx::query("INSERT INTO ingredients (name, pending_review) VALUES ('Okra', 1)")
        .execute(&harness.pool)
        .await
        .expect("the seed row is written");
    (tomato, onion)
}

#[tokio::test]
async fn list_has_one_row_link_per_ingredient_and_no_actions() {
    let harness = Harness::start("ingredient-list-rows").await;
    let token = admin_token(&harness).await;
    let (tomato, onion) = seed(&harness).await;

    let body = body_text(harness.get("/admin/ingredients", Some(&token)).await).await;
    assert_eq!(
        body.matches("href=\"/admin/ingredients/").count(),
        3,
        "{body}"
    );
    assert!(
        body.contains(&format!("href=\"/admin/ingredients/{tomato}\"")),
        "{body}"
    );
    assert!(
        body.contains(&format!("href=\"/admin/ingredients/{onion}\"")),
        "{body}"
    );
    for expected in ["Produce", "Alternative names: 1", "Waiting for review"] {
        assert!(body.contains(expected), "{expected}: {body}");
    }
    assert!(!body.contains("Alternative names: 0"), "{body}");
    assert!(body.contains("class=\"badge\""), "{body}");
    assert!(!body.contains("Rename"), "{body}");
    assert!(!body.contains("Delete"), "{body}");
    assert!(!body.contains("Merge into"), "{body}");
    assert!(!body.contains("name=\"id\""), "{body}");
}

#[tokio::test]
async fn search_filters_by_name_and_alias_case_insensitively() {
    let harness = Harness::start("ingredient-search").await;
    let token = admin_token(&harness).await;
    let (tomato, onion) = seed(&harness).await;

    let body = body_text(harness.get("/admin/ingredients?q=POMO", Some(&token)).await).await;
    assert!(
        body.contains(&format!("href=\"/admin/ingredients/{tomato}\"")),
        "{body}"
    );
    assert!(
        !body.contains(&format!("href=\"/admin/ingredients/{onion}\"")),
        "{body}"
    );
    assert!(body.contains("value=\"POMO\""), "{body}");

    let body = body_text(
        harness
            .get("/admin/ingredients?q=onion", Some(&token))
            .await,
    )
    .await;
    assert!(
        body.contains(&format!("href=\"/admin/ingredients/{onion}\"")),
        "{body}"
    );
    assert!(
        !body.contains(&format!("href=\"/admin/ingredients/{tomato}\"")),
        "{body}"
    );

    let body = body_text(harness.get("/admin/ingredients?q=zzz", Some(&token)).await).await;
    assert!(body.contains("No ingredients match your search."), "{body}");
    assert!(!body.contains("href=\"/admin/ingredients/"), "{body}");
}

#[tokio::test]
async fn detail_page_shows_the_ingredient_actions() {
    let harness = Harness::start("ingredient-detail").await;
    let token = admin_token(&harness).await;
    let (tomato, _) = seed(&harness).await;

    let response = harness
        .get(&format!("/admin/ingredients/{tomato}"), Some(&token))
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    for expected in [
        "<h1>Tomato</h1>",
        "href=\"/admin/ingredients\"",
        "Rename",
        "Pomodoro",
        "Store section",
        "Save",
    ] {
        assert!(body.contains(expected), "{expected}: {body}");
    }
}

#[tokio::test]
async fn detail_page_without_confirm_has_only_the_links() {
    let harness = Harness::start("ingredient-links").await;
    let token = admin_token(&harness).await;
    let (tomato, _) = seed(&harness).await;

    let body = body_text(
        harness
            .get(&format!("/admin/ingredients/{tomato}"), Some(&token))
            .await,
    )
    .await;
    assert!(
        body.contains(&format!(
            "href=\"/admin/ingredients/{tomato}?confirm=delete\""
        )),
        "{body}"
    );
    assert!(
        body.contains(&format!(
            "href=\"/admin/ingredients/{tomato}?confirm=merge\""
        )),
        "{body}"
    );
    assert!(!body.contains("class=\"danger\""), "{body}");
    assert!(!body.contains("name=\"from\""), "{body}");
    assert!(!body.contains("Delete Tomato?"), "{body}");
}

#[tokio::test]
async fn confirm_delete_shows_the_question_and_the_delete_form() {
    let harness = Harness::start("ingredient-confirm-delete").await;
    let token = admin_token(&harness).await;
    let (tomato, _) = seed(&harness).await;

    let body = body_text(
        harness
            .get(
                &format!("/admin/ingredients/{tomato}?confirm=delete"),
                Some(&token),
            )
            .await,
    )
    .await;
    assert!(body.contains("Delete Tomato?"), "{body}");
    assert!(body.contains("class=\"danger\""), "{body}");
    assert!(
        body.contains(&format!("href=\"/admin/ingredients/{tomato}\"")),
        "{body}"
    );
    assert!(!body.contains("name=\"from\""), "{body}");
}

#[tokio::test]
async fn confirm_merge_shows_the_question_and_the_merge_form() {
    let harness = Harness::start("ingredient-confirm-merge").await;
    let token = admin_token(&harness).await;
    let (tomato, onion) = seed(&harness).await;

    let body = body_text(
        harness
            .get(
                &format!("/admin/ingredients/{tomato}?confirm=merge"),
                Some(&token),
            )
            .await,
    )
    .await;
    assert!(
        body.contains("Merge Tomato into another ingredient?"),
        "{body}"
    );
    assert!(body.contains("name=\"from\""), "{body}");
    assert!(
        body.contains(&format!("<option value=\"{onion}\">Onion</option>")),
        "{body}"
    );
    assert!(
        body.contains(&format!("href=\"/admin/ingredients/{tomato}\"")),
        "{body}"
    );
    assert!(!body.contains("class=\"danger\""), "{body}");
}

#[tokio::test]
async fn unknown_ingredient_says_it_does_not_exist() {
    let harness = Harness::start("ingredient-missing").await;
    let token = admin_token(&harness).await;
    let body = body_text(harness.get("/admin/ingredients/9999", Some(&token)).await).await;
    assert!(body.contains("That item does not exist."), "{body}");
    assert!(!body.contains("Rename"), "{body}");
}

#[tokio::test]
async fn member_sees_the_not_allowed_message_on_the_ingredient_page() {
    let harness = Harness::start("ingredient-member").await;
    harness.issuer.set_groups(&[]);
    let token = session_token(&harness.sign_in().await).expect("sign-in sets a session");
    let (tomato, _) = seed(&harness).await;
    let response = harness
        .get(&format!("/admin/ingredients/{tomato}"), Some(&token))
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(
        body.contains("You are not allowed to manage the catalog."),
        "{body}"
    );
    assert!(!body.contains("Rename"), "{body}");
}
