//! A recipe with something the household avoids is flagged on the list and its page, and the
//! "Safe for us" filter leaves it out.

use axum::http::StatusCode;
use crabbage_domain::allergens::Allergen;
use crabbage_store::catalog::allergens::set_allergens;
use crabbage_store::catalog::ingredients;
use crabbage_store::planner::{Avoid, save_avoid};
use crabbage_store::recipes::{LineInput, RecipeInput, create};

use crate::common::{Harness, body_text, sign_in_as};

const GROUP: &str = "crabbage-household-avoid-home";

async fn page(harness: &Harness, token: &str, path: &str) -> String {
    let response = harness.get(path, Some(token)).await;
    assert_eq!(response.status(), StatusCode::OK, "{path}");
    body_text(response).await.replace("<!>", "")
}

fn recipe(title: &str, ingredient: &str) -> RecipeInput {
    RecipeInput {
        title: title.to_owned(),
        servings: 2,
        lines: vec![LineInput {
            amount: None,
            unit: None,
            ingredient: Some(ingredient.to_owned()),
            note: None,
            section_id: None,
            allergens: Vec::new(),
        }],
        steps: Vec::new(),
        category_ids: Vec::new(),
        meal_types: Vec::new(),
        warnings: Vec::new(),
    }
}

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn setup(harness: &Harness) -> (String, i64, i64) {
    let token = sign_in_as(harness, "avoid-cook", &[GROUP]).await;
    let (owner, household): (i64, i64) = sqlx::query_as(
        "SELECT u.id, m.household_id FROM users u JOIN user_households m ON m.user_id = u.id \
         WHERE u.subject = 'avoid-cook'",
    )
    .fetch_one(&harness.pool)
    .await
    .expect("member exists");
    let pesto = ingredients::create(&harness.pool, "Pesto")
        .await
        .expect("ingredient created");
    set_allergens(&harness.pool, pesto, &[Allergen::TreeNuts])
        .await
        .expect("allergens set");
    let avoid = Avoid {
        allergens: vec![Allergen::TreeNuts],
        warnings: Vec::new(),
    };
    save_avoid(&harness.pool, household, &avoid)
        .await
        .expect("avoid saved");
    let pasta = create(&harness.pool, owner, &recipe("Pasta", "Pesto"), 0)
        .await
        .expect("recipe created");
    let toast = create(&harness.pool, owner, &recipe("Toast", "Bread"), 0)
        .await
        .expect("recipe created");
    (token, pasta, toast)
}

#[tokio::test]
async fn avoided_recipes_are_flagged_and_the_safe_filter_leaves_them_out() {
    let harness = Harness::start("recipe-avoid").await;
    let (token, pasta, toast) = setup(&harness).await;

    let body = page(&harness, &token, "/recipes").await;
    assert_eq!(
        body.matches("class=\"recipe-card-avoided\"").count(),
        1,
        "{body}"
    );
    assert!(body.contains("Pasta") && body.contains("Toast"), "{body}");
    let body = page(&harness, &token, "/recipes?safe=1").await;
    assert!(body.contains("Toast") && !body.contains("Pasta"), "{body}");
    assert!(
        body.contains("href=\"/recipes\" aria-current=\"page\">Safe for us</a>"),
        "{body}"
    );

    let body = page(&harness, &token, &format!("/recipes/{pasta}")).await;
    assert!(
        body.contains("<strong>Your household avoids:</strong> Tree nuts</p>"),
        "{body}"
    );
    let body = page(&harness, &token, &format!("/recipes/{toast}")).await;
    assert!(!body.contains("recipe-avoided"), "{body}");
}
