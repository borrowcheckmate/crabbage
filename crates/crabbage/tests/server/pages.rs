//! The admin pages render for admins, show a message to everyone else, and the home page links to
//! them only for admins.

use axum::http::StatusCode;

use crabbage_store::catalog::{categories, ingredients, sections};

use crate::common::{Harness, body_text, session_token};

async fn token_for(harness: &Harness, groups: &[&str]) -> String {
    harness.issuer.set_groups(groups);
    session_token(&harness.sign_in().await).expect("sign-in sets a session")
}

#[tokio::test]
async fn admin_sees_the_ingredients_page() {
    let harness = Harness::start("pages-admin").await;
    let token = token_for(&harness, &["crabbage-admin"]).await;
    let response = harness.get("/admin/ingredients", Some(&token)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("Ingredients</h1>"), "{body}");
    assert!(!body.contains("not allowed"), "{body}");
}

#[tokio::test]
async fn member_sees_the_not_allowed_message_on_admin_pages() {
    let harness = Harness::start("pages-member").await;
    let token = token_for(&harness, &[]).await;
    for path in [
        "/admin/ingredients",
        "/admin/sections",
        "/admin/categories",
        "/admin/review",
        "/admin/mealie",
    ] {
        let response = harness.get(path, Some(&token)).await;
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        let body = body_text(response).await;
        assert!(
            body.contains("You are not allowed to manage the catalog."),
            "{path}: {body}"
        );
        assert!(!body.contains("Ingredients</h1>"), "{path}: {body}");
    }
}

#[tokio::test]
async fn home_links_to_the_admin_pages_only_for_admins() {
    let harness = Harness::start("pages-home").await;
    let admin = token_for(&harness, &["crabbage-admin"]).await;
    let body = body_text(harness.get("/", Some(&admin)).await).await;
    assert!(body.contains("href=\"/admin/ingredients\""), "{body}");
    assert!(body.contains("href=\"/admin/categories\""), "{body}");
    assert!(body.contains("href=\"/admin/mealie\""), "{body}");

    let member = token_for(&harness, &[]).await;
    let body = body_text(harness.get("/", Some(&member)).await).await;
    assert!(!body.contains("/admin/ingredients"), "{body}");
    assert!(!body.contains("/admin/mealie"), "{body}");
}

#[tokio::test]
async fn admin_mealie_page_has_a_password_token_form_and_summary_labels() {
    let harness = Harness::start("pages-mealie").await;
    let token = token_for(&harness, &["crabbage-admin"]).await;
    let body = body_text(harness.get("/admin/mealie", Some(&token)).await).await;
    for expected in [
        "Import from Mealie</h1>",
        "name=\"url\"",
        "name=\"token\"",
        "type=\"password\"",
    ] {
        assert!(body.contains(expected), "{expected}: {body}");
    }
}

#[tokio::test]
async fn member_cannot_start_a_mealie_migration() {
    let harness = Harness::start("pages-mealie-member-start").await;
    let token = token_for(&harness, &[]).await;
    let response = harness
        .post_form(
            "/admin/mealie/start",
            Some(&token),
            "url=http%3A%2F%2F127.0.0.1%3A1&token=invalid",
        )
        .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM recipes")
        .fetch_one(&harness.pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn ingredients_page_lists_sections_alias_counts_and_pending_ingredients() {
    let harness = Harness::start("pages-ingredients").await;
    let token = token_for(&harness, &["crabbage-admin"]).await;
    let produce = sections::create(&harness.pool, "Produce").await.unwrap();
    let tomato = ingredients::create(&harness.pool, "Tomato").await.unwrap();
    ingredients::create(&harness.pool, "Onion").await.unwrap();
    ingredients::set_section(&harness.pool, tomato, Some(produce))
        .await
        .unwrap();
    ingredients::add_alias(&harness.pool, tomato, "Pomodoro")
        .await
        .unwrap();
    sqlx::query("INSERT INTO ingredients (name, pending_review) VALUES ('Okra', 1)")
        .execute(&harness.pool)
        .await
        .unwrap();

    let body = body_text(harness.get("/admin/ingredients", Some(&token)).await).await;
    for expected in [
        "Tomato",
        "Produce",
        "Alternative names: 1",
        "Okra",
        "Waiting for review",
        "class=\"badge\"",
    ] {
        assert!(body.contains(expected), "{expected}: {body}");
    }
    assert!(!body.contains("Pomodoro"), "{body}");
    assert!(!body.contains("Merge into"), "{body}");
}

#[tokio::test]
async fn empty_admin_pages_say_so() {
    let harness = Harness::start("pages-empty").await;
    let token = token_for(&harness, &["crabbage-admin"]).await;
    let cases = [
        ("/admin/ingredients", "No ingredients yet."),
        ("/admin/sections", "No store sections yet."),
        ("/admin/review", "Nothing is waiting for review."),
    ];
    for (path, text) in cases {
        let body = body_text(harness.get(path, Some(&token)).await).await;
        assert!(body.contains(text), "{path}: {body}");
    }
}

#[tokio::test]
async fn sections_and_review_pages_list_their_rows() {
    let harness = Harness::start("pages-rows").await;
    let token = token_for(&harness, &["crabbage-admin"]).await;
    sections::create(&harness.pool, "Dairy").await.unwrap();
    sqlx::query("INSERT INTO ingredients (name, pending_review) VALUES ('Okra', 1)")
        .execute(&harness.pool)
        .await
        .unwrap();

    let body = body_text(harness.get("/admin/sections", Some(&token)).await).await;
    assert!(body.contains("Dairy") && body.contains("Delete"), "{body}");

    let body = body_text(harness.get("/admin/review", Some(&token)).await).await;
    assert!(body.contains("Okra") && body.contains("Approve"), "{body}");
}

#[tokio::test]
async fn categories_page_lists_the_categories_for_admins() {
    let harness = Harness::start("pages-categories").await;
    let token = token_for(&harness, &["crabbage-admin"]).await;
    categories::create(&harness.pool, "Breakfast")
        .await
        .unwrap();
    let body = body_text(harness.get("/admin/categories", Some(&token)).await).await;
    assert!(body.contains("Categories</h1>"), "{body}");
    assert!(body.contains("Breakfast"), "{body}");
    assert!(!body.contains("not allowed"), "{body}");
}
