//! The recipe pages render: the list, a recipe with its lines and numbered steps, and the forms. Edit
//! and Delete show only to the users who may use them.

use axum::http::StatusCode;

use crate::common::{Harness, body_text, sign_in_as};
use crate::recipes::create;
use crabbage_store::recipes::{LineInput, RecipeInput};

const OWNER: &str = "owner";
const OTHER: &str = "other";
const EDITOR: &str = "editor";

async fn page(harness: &Harness, token: &str, path: &str) -> String {
    let response = harness.get(path, Some(token)).await;
    assert_eq!(response.status(), StatusCode::OK, "{path}");
    body_text(response).await
}

#[tokio::test]
async fn the_list_links_each_recipe_and_offers_a_new_one() {
    let harness = Harness::start("recipe-pages-list").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    let other = sign_in_as(&harness, OTHER, &[]).await;

    let body = page(&harness, &other, "/recipes").await;
    assert!(body.contains("Recipes</h1>"), "{body}");
    assert!(body.contains(&format!("href=\"/recipes/{id}\"")), "{body}");
    assert!(body.contains("Pancakes"), "{body}");
    assert!(body.contains("New recipe"), "{body}");
}

#[tokio::test]
async fn the_home_page_links_to_recipes_for_every_member() {
    let harness = Harness::start("recipe-pages-home").await;
    let member = sign_in_as(&harness, OTHER, &[]).await;
    let body = page(&harness, &member, "/").await;
    assert!(body.contains("href=\"/recipes\""), "{body}");
    assert!(!body.contains("/admin/ingredients"), "{body}");
}

#[tokio::test]
async fn a_recipe_page_shows_its_lines_and_numbered_steps() {
    let harness = Harness::start("recipe-pages-detail").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;

    let body = page(&harness, &owner, &format!("/recipes/{id}")).await;
    assert!(body.contains("Pancakes</h1>"), "{body}");
    for expected in [
        "250 g Flour, sifted",
        "<li>2 Egg</li>",
        "<li>Salt, to taste</li>",
        "<ol>",
        "Whisk the eggs",
        "2 min",
        "Fry until golden",
    ] {
        assert!(body.contains(expected), "{expected}: {body}");
    }
    assert!(!body.contains("piece"), "{body}");
}

#[tokio::test]
async fn a_dutch_recipe_page_uses_dutch_words_and_a_decimal_comma() {
    let harness = Harness::start_with_language("recipe-pages-nl", Some("nl")).await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let owner_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE subject = ?")
        .bind(OWNER)
        .fetch_one(&harness.pool)
        .await
        .expect("the user exists");
    let line = |amount: f64, unit: &str, ingredient: &str| LineInput {
        amount: Some(amount),
        unit: Some(unit.to_owned()),
        ingredient: Some(ingredient.to_owned()),
        note: None,
    };
    let input = RecipeInput {
        title: "Pannenkoeken".to_owned(),
        servings: 4,
        lines: vec![
            line(1.25, "tbsp", "Suiker"),
            line(0.5, "tsp", "Vanille"),
            line(2.0, "piece", "Ei"),
            LineInput {
                amount: None,
                unit: Some("to taste".to_owned()),
                ingredient: Some("Zout".to_owned()),
                note: None,
            },
        ],
        steps: Vec::new(),
    };
    let id = crabbage_store::recipes::create(&harness.pool, owner_id, &input, 0)
        .await
        .expect("recipe is stored");

    let body = page(&harness, &owner, &format!("/recipes/{id}")).await;
    for expected in [
        "1,25 el Suiker",
        "0,5 tl Vanille",
        "2 Ei",
        "Zout, naar smaak",
    ] {
        assert!(body.contains(expected), "{expected}: {body}");
    }
}

#[tokio::test]
async fn edit_and_delete_show_only_to_those_who_may_use_them() {
    let harness = Harness::start("recipe-pages-buttons").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    let edit_link = format!("href=\"/recipes/{id}/edit\"");

    let body = page(&harness, &owner, &format!("/recipes/{id}")).await;
    assert!(body.contains(&edit_link), "{body}");
    assert!(body.contains("class=\"danger\""), "{body}");

    let other = sign_in_as(&harness, OTHER, &[]).await;
    let body = page(&harness, &other, &format!("/recipes/{id}")).await;
    assert!(!body.contains(&edit_link), "{body}");
    assert!(!body.contains("class=\"danger\""), "{body}");

    let admin = sign_in_as(&harness, "admin", &["crabbage-admin"]).await;
    let body = page(&harness, &admin, &format!("/recipes/{id}")).await;
    assert!(body.contains(&edit_link), "{body}");
    assert!(body.contains("class=\"danger\""), "{body}");
}

#[tokio::test]
async fn the_new_form_offers_the_fields_and_the_add_buttons() {
    let harness = Harness::start("recipe-pages-new").await;
    let member = sign_in_as(&harness, OWNER, &[]).await;
    let body = page(&harness, &member, "/recipes/new").await;
    assert!(body.contains("New recipe</h1>"), "{body}");
    for expected in ["Add ingredient", "Add step", "ingredient-names", "Save"] {
        assert!(body.contains(expected), "{expected}: {body}");
    }
    assert!(!body.contains("Who may edit this recipe"), "{body}");
}

#[tokio::test]
async fn the_edit_page_shows_the_editors_to_owners_and_admins_only() {
    let harness = Harness::start("recipe-pages-edit").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    let body = page(&harness, &owner, &format!("/recipes/{id}/edit")).await;
    assert!(body.contains("Edit recipe</h1>"), "{body}");
    assert!(body.contains("Add ingredient"), "{body}");
    assert!(body.contains("Who may edit this recipe"), "{body}");
    assert!(body.contains("Pancakes"), "{body}");

    let other = sign_in_as(&harness, OTHER, &[]).await;
    let body = page(&harness, &other, &format!("/recipes/{id}/edit")).await;
    assert!(body.contains("not allowed to change this recipe"), "{body}");
    assert!(!body.contains("Add ingredient"), "{body}");

    let editor = sign_in_as(&harness, EDITOR, &[]).await;
    let editor_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE subject = ?")
        .bind(EDITOR)
        .fetch_one(&harness.pool)
        .await
        .expect("the user exists");
    crabbage_store::recipes::add_editor(&harness.pool, id, editor_id)
        .await
        .expect("editor is added");
    let body = page(&harness, &editor, &format!("/recipes/{id}/edit")).await;
    assert!(body.contains("Add ingredient"), "{body}");
    assert!(!body.contains("Who may edit this recipe"), "{body}");
}

#[tokio::test]
async fn an_unknown_recipe_says_it_does_not_exist() {
    let harness = Harness::start("recipe-pages-missing").await;
    let member = sign_in_as(&harness, OWNER, &[]).await;
    let body = page(&harness, &member, "/recipes/999").await;
    assert!(body.contains("does not exist"), "{body}");
}
