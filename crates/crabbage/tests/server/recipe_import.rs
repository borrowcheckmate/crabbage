//! Importing a recipe from a web page: the page asks for an address, shows the recipe read from it in
//! the recipe form with the unresolved lines marked, and saving creates the recipe for the importer
//! with the photo downloaded.

use axum::Router;
use axum::http::StatusCode;
use axum::http::header::{CONTENT_TYPE, LOCATION};
use axum::response::Html;
use axum::routing::get;
use crabbage_store::catalog::ingredients;
use crabbage_store::recipes::get as get_recipe;
use serde_json::json;
use tokio::net::TcpListener;

use crate::common::{Harness, body_text, header, sign_in_as};
use crate::recipes::call;

const MEMBER: &str = "importer";
const ALLOW: &[(&str, &str)] = &[("CRABBAGE_IMPORT_ALLOW_PRIVATE", "true")];
const PNG: &[u8] = b"\x89PNG\r\n\x1a\nimage";

fn recipe_page(base: &str) -> String {
    let page = json!({
        "@context": "https://schema.org",
        "@type": "Recipe",
        "name": "Okra Pancakes",
        "image": format!("{base}/photo.png"),
        "recipeYield": "4 servings",
        "recipeIngredient": ["200 g Flour", "2 Okra pods, sliced"],
        "recipeInstructions": [{"@type": "HowToStep", "text": "Stir the flour with the okra pods."}]
    });
    format!(
        "<html lang=\"en\"><head><script type=\"application/ld+json\">{page}</script></head></html>"
    )
}

/// Serves a recipe page, a page without a recipe, a PNG and a 404 on a loopback port, and returns
/// the base address.
async fn serve() -> String {
    serve_with(recipe_page).await
}

/// Like [`serve`], with `recipe` building the recipe page from the base address.
pub(crate) async fn serve_with(recipe: fn(&str) -> String) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let base = format!("http://{}", listener.local_addr().expect("address"));
    let page = recipe(&base);
    let router = Router::new()
        .route("/recipe", get(move || async move { Html(page) }))
        .route(
            "/plain",
            get(|| async { Html("<html><body>No data</body></html>") }),
        )
        .route(
            "/photo.png",
            get(|| async { ([(CONTENT_TYPE, "image/png")], PNG) }),
        )
        .route("/notes.png", get(|| async { "just text" }));
    tokio::spawn(async move { drop(axum::serve(listener, router).await) });
    base
}

async fn importer(harness: &Harness) -> i64 {
    sqlx::query_scalar("SELECT id FROM users WHERE subject = ?")
        .bind(MEMBER)
        .fetch_one(&harness.pool)
        .await
        .expect("the user exists")
}

async fn page(harness: &Harness, token: &str, path: &str) -> String {
    let response = harness.get(path, Some(token)).await;
    assert_eq!(response.status(), StatusCode::OK, "{path}");
    body_text(response).await
}

#[tokio::test]
async fn the_import_page_asks_for_an_address_and_the_list_links_to_it() {
    let harness = Harness::start("import-form").await;
    let token = sign_in_as(&harness, MEMBER, &[]).await;
    let body = page(&harness, &token, "/recipes/import").await;
    assert!(body.contains("Import a recipe"), "{body}");
    assert!(body.contains("action=\"/recipes/import\""), "{body}");
    assert!(body.contains("name=\"url\""), "{body}");
    assert!(!body.contains("unresolved"), "{body}");
    let list = page(&harness, &token, "/recipes").await;
    assert!(list.contains("href=\"/recipes/import\""), "{list}");
}

#[tokio::test]
async fn signed_out_users_are_sent_to_sign_in() {
    let harness = Harness::start("import-signed-out").await;
    let response = harness.get("/recipes/import", None).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(header(&response, LOCATION).as_deref(), Some("/auth/login"));
}

#[tokio::test]
async fn the_preview_fills_the_form_and_marks_the_unresolved_line() {
    let harness = Harness::start_with("import-preview", None, ALLOW).await;
    ingredients::create(&harness.pool, "Flour")
        .await
        .expect("ingredient");
    let token = sign_in_as(&harness, MEMBER, &[]).await;
    let base = serve().await;
    let body = page(
        &harness,
        &token,
        &format!("/recipes/import?url={base}/recipe"),
    )
    .await;
    assert!(body.contains("Okra Pancakes"), "{body}");
    assert!(body.contains("value=\"Flour\""), "{body}");
    assert!(body.contains("value=\"Okra pods\""), "{body}");
    assert!(body.contains("value=\"sliced\""), "{body}");
    assert!(body.contains("value=\"g\" selected"), "{body}");
    assert!(body.contains("Note"), "{body}");
    assert_eq!(body.matches("card unresolved").count(), 1, "{body}");
    assert!(
        body.contains("Not found among the ingredients: Okra pods"),
        "{body}"
    );
    assert!(body.contains("AI is off"), "{body}");
    assert!(
        body.contains(&format!("src=\"{base}/photo.png\"")),
        "{body}"
    );
    assert!(
        body.contains(&format!("name=\"image\" value=\"{base}/photo.png\"")),
        "{body}"
    );
}

#[tokio::test]
async fn an_ai_that_fails_is_reported_on_the_preview() {
    let extra = [
        ("CRABBAGE_IMPORT_ALLOW_PRIVATE", "true"),
        ("AI_BASE_URL", "http://127.0.0.1:1/v1"),
        ("AI_MODEL", "small"),
    ];
    let harness = Harness::start_with("import-ai", None, &extra).await;
    let token = sign_in_as(&harness, MEMBER, &[]).await;
    let base = serve().await;
    let body = page(
        &harness,
        &token,
        &format!("/recipes/import?url={base}/recipe"),
    )
    .await;
    assert!(
        body.contains("The AI could not complete this recipe"),
        "{body}"
    );
    assert!(!body.contains("AI is off"), "{body}");
    assert!(body.contains("Okra Pancakes"), "{body}");
}

#[tokio::test]
async fn a_page_without_a_recipe_shows_a_clear_message() {
    let harness = Harness::start_with("import-plain", None, ALLOW).await;
    let token = sign_in_as(&harness, MEMBER, &[]).await;
    let base = serve().await;
    let body = page(
        &harness,
        &token,
        &format!("/recipes/import?url={base}/plain"),
    )
    .await;
    assert!(
        body.contains("The page has no recipe data to import."),
        "{body}"
    );
}

#[tokio::test]
async fn a_private_address_is_refused_unless_allowed() {
    let harness = Harness::start("import-private").await;
    let token = sign_in_as(&harness, MEMBER, &[]).await;
    let base = serve().await;
    let body = page(
        &harness,
        &token,
        &format!("/recipes/import?url={base}/recipe"),
    )
    .await;
    assert!(
        body.contains("That address is on a private network"),
        "{body}"
    );
    assert!(!body.contains("Okra Pancakes"), "{body}");
}

fn edited_draft() -> String {
    json!({
        "title": "Okra Pancakes",
        "servings": "4",
        "lines": [
            {"amount": "200", "unit": "g", "ingredient": "Flour", "note": ""},
            {"amount": "3", "unit": "piece", "ingredient": "Okra", "note": "smaller than the page says"}
        ],
        "steps": [{"text": "Stir.", "minutes": "", "line_positions": [0, 1]}]
    })
    .to_string()
}

#[tokio::test]
async fn saving_creates_the_recipe_for_the_importer_with_their_edits_and_the_photo() {
    let harness = Harness::start_with("import-save", None, ALLOW).await;
    ingredients::create(&harness.pool, "Flour")
        .await
        .expect("ingredient");
    let token = sign_in_as(&harness, MEMBER, &[]).await;
    let base = serve().await;
    let (status, body) = call(
        &harness,
        &token,
        "save_import",
        &[
            ("draft", &edited_draft()),
            ("image", &format!("{base}/photo.png")),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let id: i64 = body.trim().parse().expect("the id");
    let user = importer(&harness).await;
    let recipe = get_recipe(&harness.pool, id, user)
        .await
        .expect("read")
        .expect("recipe");
    assert_eq!(recipe.owner_id, user);
    assert_eq!(recipe.lines[1].ingredient.as_deref(), Some("Okra"));
    assert_eq!(recipe.lines[1].unit.as_deref(), Some("piece"));
    assert_eq!(
        recipe.lines[1].note.as_deref(),
        Some("smaller than the page says")
    );
    let pending: Vec<String> = ingredients::list_pending(&harness.pool)
        .await
        .expect("pending")
        .into_iter()
        .map(|ingredient| ingredient.name)
        .collect();
    assert_eq!(pending, ["Okra"]);
    let name = recipe.image.expect("the photo is attached");
    assert_eq!(
        std::fs::read(harness.images_dir().join(name)).expect("file"),
        PNG
    );
}

#[tokio::test]
async fn a_photo_that_cannot_be_downloaded_does_not_fail_the_save() {
    let harness = Harness::start_with("import-bad-photo", None, ALLOW).await;
    let token = sign_in_as(&harness, MEMBER, &[]).await;
    let base = serve().await;
    for image in ["/missing.png", "/notes.png"] {
        let (status, body) = call(
            &harness,
            &token,
            "save_import",
            &[
                ("draft", &edited_draft()),
                ("image", &format!("{base}{image}")),
            ],
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let id: i64 = body.trim().parse().expect("the id");
        let recipe = get_recipe(&harness.pool, id, importer(&harness).await)
            .await
            .expect("read")
            .expect("recipe");
        assert_eq!(recipe.image, None, "{image}");
    }
    assert!(
        !harness.images_dir().exists()
            || std::fs::read_dir(harness.images_dir())
                .expect("dir")
                .count()
                == 0
    );
}
