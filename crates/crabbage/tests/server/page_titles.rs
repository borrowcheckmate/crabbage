//! Every page names itself before the app in the document title, and the icon is served to
//! everyone, signed in or not.

use axum::http::StatusCode;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};

use crate::common::{Harness, body_text, header, sign_in_as};
use crate::recipes::create;

const HOME: &str = "crabbage-household-home";

async fn title(harness: &Harness, token: &str, path: &str) -> String {
    let body = body_text(harness.get(path, Some(token)).await).await;
    let (_, rest) = body.split_once("<title>").expect("the page has a title");
    let (title, _) = rest.split_once("</title>").expect("the title is closed");
    title.to_owned()
}

#[tokio::test]
async fn each_page_is_named_before_the_app() {
    let harness = Harness::start("titles-pages").await;
    let token = sign_in_as(&harness, "cook", &["admin", HOME]).await;
    for (path, expected) in [
        ("/", "Today · crabbage"),
        ("/shopping", "Shopping list · crabbage"),
        ("/plan", "Weekly plan · crabbage"),
        ("/recipes", "Recipes · crabbage"),
        ("/recipes?category=1", "Recipes · crabbage"),
        ("/recipes/new", "New recipe · crabbage"),
        ("/recipes/import", "Import a recipe · crabbage"),
        ("/admin/ingredients", "Ingredients · crabbage"),
        ("/admin/sections", "Store sections · crabbage"),
        ("/admin/settings", "AI settings · crabbage"),
        ("/nowhere", "Page not found · crabbage"),
    ] {
        assert_eq!(title(&harness, &token, path).await, expected, "{path}");
    }
}

#[tokio::test]
async fn a_recipe_page_is_named_after_its_recipe() {
    let harness = Harness::start("titles-recipe").await;
    let token = sign_in_as(&harness, "cook", &[HOME]).await;
    let id = create(&harness, &token, "Pancakes").await;
    let recipe = title(&harness, &token, &format!("/recipes/{id}")).await;
    assert_eq!(recipe, "Recipes · crabbage");
    let edit = title(&harness, &token, &format!("/recipes/{id}/edit")).await;
    assert_eq!(edit, "Edit recipe · crabbage");
}

#[tokio::test]
async fn the_icon_is_linked_and_served_without_a_session() {
    let harness = Harness::start("titles-icon").await;
    let token = sign_in_as(&harness, "cook", &[HOME]).await;
    let page = body_text(harness.get("/", Some(&token)).await).await;
    assert!(
        page.contains(r#"<link rel="icon" type="image/svg+xml" href="/favicon.svg">"#),
        "{page}"
    );
    let response = harness.get("/favicon.svg", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        header(&response, CONTENT_TYPE).as_deref(),
        Some("image/svg+xml")
    );
    assert!(header(&response, CACHE_CONTROL).is_some_and(|value| value.contains("max-age")));
    let icon = body_text(response).await;
    assert!(icon.starts_with("<svg"), "{icon}");
}
