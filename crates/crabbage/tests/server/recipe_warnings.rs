//! The recipe page names a recipe's dietary warnings and the form checks them; a recipe without any
//! shows no line.

use axum::http::StatusCode;
use crabbage_domain::warnings::Warning;
use crabbage_store::recipes::{RecipeInput, create};

use crate::common::{Harness, body_text, sign_in_as};

async fn page(harness: &Harness, token: &str, path: &str) -> String {
    let response = harness.get(path, Some(token)).await;
    assert_eq!(response.status(), StatusCode::OK, "{path}");
    body_text(response).await.replace("<!>", "")
}

fn recipe(title: &str, warnings: Vec<Warning>) -> RecipeInput {
    RecipeInput {
        title: title.to_owned(),
        servings: 2,
        lines: Vec::new(),
        steps: Vec::new(),
        category_ids: Vec::new(),
        meal_types: Vec::new(),
        warnings,
    }
}

#[tokio::test]
async fn the_recipe_page_names_the_warnings_and_the_form_checks_them() {
    let harness = Harness::start("recipe-warnings").await;
    let token = sign_in_as(&harness, "warning-cook", &[]).await;
    let owner: i64 = sqlx::query_scalar("SELECT id FROM users WHERE subject = 'warning-cook'")
        .fetch_one(&harness.pool)
        .await
        .expect("user exists");
    let tiramisu = recipe("Tiramisu", vec![Warning::RawEgg, Warning::Alcohol]);
    let tiramisu = create(&harness.pool, owner, &tiramisu, 0)
        .await
        .expect("recipe created");
    let toast = create(&harness.pool, owner, &recipe("Toast", Vec::new()), 0)
        .await
        .expect("recipe created");

    let body = page(&harness, &token, &format!("/recipes/{tiramisu}")).await;
    assert!(
        body.contains("<strong>Not advised in pregnancy:</strong> Raw egg, Alcohol</p>"),
        "{body}"
    );
    let body = page(&harness, &token, &format!("/recipes/{tiramisu}/edit")).await;
    assert!(body.contains("Pregnancy</legend>"), "{body}");
    assert!(body.contains("value=\"raw_egg\" checked"), "{body}");
    assert!(!body.contains("value=\"liver\" checked"), "{body}");
    let body = page(&harness, &token, &format!("/recipes/{toast}")).await;
    assert!(!body.contains("recipe-warnings"), "{body}");
}
