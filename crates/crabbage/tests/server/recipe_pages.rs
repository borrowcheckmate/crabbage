//! The recipe pages render: the list, a recipe with its lines and numbered steps, and the forms. Edit
//! and Delete show only to the users who may use them, and delete asks first.

use axum::http::StatusCode;

use crate::common::{Harness, body_text, sign_in_as};
use crate::recipes::create;
use crabbage_store::catalog::categories;
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
    assert!(body.contains("class=\"recipe-grid\""), "{body}");
    assert!(body.contains("class=\"recipe-photo-fallback\""), "{body}");
    assert!(body.contains("Find a recipe..."), "{body}");
    assert_eq!(
        body.matches("class=\"recipe-search-icon\"").count(),
        1,
        "{body}"
    );
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
        "Flour, sifted</span>",
        "class=\"recipe-ingredient-quantity\">250 g</span>",
        "<span>Egg</span>",
        "class=\"recipe-ingredient-quantity\">2</span>",
        "<span>Salt, to taste</span>",
        "recipe-back",
        "viewBox=\"0 0 24 24\"",
        "<ol>",
        "Whisk the eggs",
        "2 min",
        "Fry until golden",
        "class=\"recipe-detail-columns\"",
        "class=\"recipe-photo-fallback\"",
        "Start cooking",
        "Method",
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
        category_ids: Vec::new(),
    };
    let id = crabbage_store::recipes::create(&harness.pool, owner_id, &input, 0)
        .await
        .expect("recipe is stored");

    let body = page(&harness, &owner, &format!("/recipes/{id}")).await;
    for expected in [
        "<span>Suiker</span>",
        "class=\"recipe-ingredient-quantity\">1,25 el</span>",
        "<span>Vanille</span>",
        "class=\"recipe-ingredient-quantity\">0,5 tl</span>",
        "<span>Ei</span>",
        "class=\"recipe-ingredient-quantity\">2</span>",
        "<span>Zout, naar smaak</span>",
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
    let delete_link = format!("href=\"/recipes/{id}?confirm=delete\"");

    let body = page(&harness, &owner, &format!("/recipes/{id}")).await;
    assert!(body.contains(&edit_link), "{body}");
    assert!(body.contains(&delete_link), "{body}");
    assert!(!body.contains("class=\"danger\""), "{body}");

    let other = sign_in_as(&harness, OTHER, &[]).await;
    let body = page(&harness, &other, &format!("/recipes/{id}")).await;
    assert!(!body.contains(&edit_link), "{body}");
    assert!(!body.contains(&delete_link), "{body}");

    let admin = sign_in_as(&harness, "admin", &["admin"]).await;
    let body = page(&harness, &admin, &format!("/recipes/{id}")).await;
    assert!(body.contains(&edit_link), "{body}");
    assert!(body.contains(&delete_link), "{body}");
}

#[tokio::test]
async fn the_recipe_page_without_confirm_has_no_delete_form() {
    let harness = Harness::start("recipe-pages-no-confirm").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;

    let body = page(&harness, &owner, &format!("/recipes/{id}")).await;
    assert!(!body.contains("delete_recipe"), "{body}");
    assert!(!body.contains("Delete Pancakes?"), "{body}");
}

#[tokio::test]
async fn confirm_delete_shows_the_question_and_the_delete_form() {
    let harness = Harness::start("recipe-pages-confirm-delete").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;

    let body = page(&harness, &owner, &format!("/recipes/{id}?confirm=delete")).await;
    assert!(body.contains("Delete Pancakes?"), "{body}");
    assert!(body.contains("delete_recipe"), "{body}");
    assert!(body.contains("class=\"danger\""), "{body}");
    assert!(body.contains(&format!("href=\"/recipes/{id}\"")), "{body}");
    assert!(body.contains("Cancel"), "{body}");
    assert!(!body.contains("?confirm=delete\""), "{body}");
}

#[tokio::test]
async fn confirm_delete_is_not_shown_to_those_who_may_not_delete() {
    let harness = Harness::start("recipe-pages-confirm-other").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;

    let other = sign_in_as(&harness, OTHER, &[]).await;
    let body = page(&harness, &other, &format!("/recipes/{id}?confirm=delete")).await;
    assert!(!body.contains("Delete Pancakes?"), "{body}");
    assert!(!body.contains("delete_recipe"), "{body}");
    assert!(!body.contains("class=\"danger\""), "{body}");
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

/// Stores a recipe owned by the owner, filed under `category_ids`, and returns its id.
#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn filed_recipe(harness: &Harness, title: &str, category_ids: Vec<i64>) -> i64 {
    let owner_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE subject = ?")
        .bind(OWNER)
        .fetch_one(&harness.pool)
        .await
        .expect("the user exists");
    let input = RecipeInput {
        title: title.to_owned(),
        servings: 2,
        lines: Vec::new(),
        steps: Vec::new(),
        category_ids,
    };
    crabbage_store::recipes::create(&harness.pool, owner_id, &input, 0)
        .await
        .expect("recipe is stored")
}

#[tokio::test]
async fn the_list_shows_only_the_chosen_category_and_marks_it() {
    let harness = Harness::start("recipe-pages-filter").await;
    let member = sign_in_as(&harness, OWNER, &[]).await;
    let breakfast = categories::create(&harness.pool, "Breakfast")
        .await
        .unwrap();
    let dinner = categories::create(&harness.pool, "Dinner").await.unwrap();
    filed_recipe(&harness, "Pancakes", vec![breakfast]).await;
    filed_recipe(&harness, "Roast", vec![dinner]).await;

    let body = page(&harness, &member, &format!("/recipes?category={breakfast}")).await;
    assert!(body.contains("Pancakes</a>"), "{body}");
    assert!(!body.contains("Roast</a>"), "{body}");
    // NOTE: the chosen category plus the Recipes link in the side and bottom navigation.
    assert_eq!(body.matches("aria-current=\"page\"").count(), 3, "{body}");
    assert!(
        body.contains(&format!(
            "<a href=\"/recipes?category={breakfast}\" aria-current=\"page\""
        )),
        "{body}"
    );
    assert!(
        body.contains(&format!("href=\"/recipes?category={dinner}\"")),
        "{body}"
    );

    let body = page(&harness, &member, "/recipes").await;
    assert!(body.contains("Pancakes</a>"), "{body}");
    assert!(body.contains("Roast</a>"), "{body}");
    assert!(
        body.contains("<a href=\"/recipes\" aria-current=\"page\""),
        "{body}"
    );
}

#[tokio::test]
async fn an_unknown_category_shows_the_empty_text() {
    let harness = Harness::start("recipe-pages-unknown-category").await;
    let member = sign_in_as(&harness, OWNER, &[]).await;
    filed_recipe(&harness, "Pancakes", vec![]).await;
    let body = page(&harness, &member, "/recipes?category=999").await;
    assert!(body.contains("No recipes in this category."), "{body}");
    assert!(!body.contains("Pancakes</a>"), "{body}");
}

#[tokio::test]
async fn a_recipe_page_shows_its_categories_in_the_header() {
    let harness = Harness::start("recipe-pages-badges").await;
    let member = sign_in_as(&harness, OWNER, &[]).await;
    let breakfast = categories::create(&harness.pool, "Breakfast")
        .await
        .unwrap();
    categories::create(&harness.pool, "Dinner").await.unwrap();
    let id = filed_recipe(&harness, "Pancakes", vec![breakfast]).await;

    let body = page(&harness, &member, &format!("/recipes/{id}")).await;
    assert!(
        body.contains("class=\"recipe-detail-categories\"><span>Breakfast</span>"),
        "{body}"
    );
    assert!(!body.contains("Dinner"), "{body}");
}

#[tokio::test]
async fn the_forms_offer_a_checkbox_per_category_checked_for_the_recipe() {
    let harness = Harness::start("recipe-pages-category-boxes").await;
    let member = sign_in_as(&harness, OWNER, &[]).await;
    let breakfast = categories::create(&harness.pool, "Breakfast")
        .await
        .unwrap();
    categories::create(&harness.pool, "Dinner").await.unwrap();
    let id = filed_recipe(&harness, "Pancakes", vec![breakfast]).await;

    let body = page(&harness, &member, "/recipes/new").await;
    assert!(body.contains("Categories</legend>"), "{body}");
    assert!(body.contains("type=\"checkbox\""), "{body}");
    assert!(body.contains("Dinner"), "{body}");

    let body = page(&harness, &member, &format!("/recipes/{id}/edit")).await;
    assert!(
        body.contains(&format!("value=\"{breakfast}\" checked")),
        "{body}"
    );
    assert!(body.contains("Dinner"), "{body}");
}

#[tokio::test]
async fn the_edit_page_offers_each_line_to_each_step_with_the_linked_ones_checked() {
    let harness = Harness::start("recipe-pages-step-lines").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    let body = page(&harness, &owner, &format!("/recipes/{id}/edit")).await;
    assert_eq!(
        body.matches("Ingredients for this step").count(),
        3,
        "{body}"
    );
    assert_eq!(body.matches("type=\"checkbox\"").count(), 9, "{body}");
    assert_eq!(body.matches("checked>2 Egg</label>").count(), 1, "{body}");
    assert_eq!(
        body.matches("checked>250 g Flour, sifted</label>").count(),
        1,
        "{body}"
    );
    assert_eq!(
        body.matches("checked>Salt, to taste</label>").count(),
        1,
        "{body}"
    );
    assert!(body.contains("&quot;line_positions&quot;:[1]"), "{body}");
}

#[tokio::test]
async fn the_new_form_shows_no_step_ingredients_without_lines() {
    let harness = Harness::start("recipe-pages-step-lines-new").await;
    let member = sign_in_as(&harness, OWNER, &[]).await;
    let body = page(&harness, &member, "/recipes/new").await;
    assert!(!body.contains("Ingredients for this step"), "{body}");
}
