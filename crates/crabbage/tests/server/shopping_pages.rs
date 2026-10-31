use axum::http::StatusCode;
use crabbage_store::catalog::{ingredients, sections};
use crabbage_store::households::households_of;
use crabbage_store::shopping::add_text;

use crate::common::{Harness, body_text, sign_in_as};

const HOME: &str = "crabbage-household-shopping-page-home";

async fn list_context(harness: &Harness) -> (String, i64) {
    let token = sign_in_as(harness, "shopper", &[HOME]).await;
    let user: i64 = sqlx::query_scalar("SELECT id FROM users WHERE subject = 'shopper'")
        .fetch_one(&harness.pool)
        .await
        .expect("shopper exists");
    let household = households_of(&harness.pool, user)
        .await
        .expect("households")
        .first()
        .expect("household")
        .id;
    (token, household)
}

#[tokio::test]
async fn list_groups_items_in_section_order_and_posts_checked_forms() {
    let harness = Harness::start("shopping-page-groups").await;
    let (token, household) = list_context(&harness).await;
    let produce = sections::create(&harness.pool, "Produce")
        .await
        .expect("section");
    let dairy = sections::create(&harness.pool, "Dairy")
        .await
        .expect("section");
    let tomato = ingredients::create(&harness.pool, "Tomato")
        .await
        .expect("ingredient");
    let cheese = ingredients::create(&harness.pool, "Cheese")
        .await
        .expect("ingredient");
    ingredients::set_section(&harness.pool, tomato, Some(produce))
        .await
        .expect("section");
    ingredients::set_section(&harness.pool, cheese, Some(dairy))
        .await
        .expect("section");
    for ingredient in [tomato, cheese] {
        sqlx::query("INSERT INTO shopping_items (household_id, ingredient_id) VALUES (?, ?)")
            .bind(household)
            .bind(ingredient)
            .execute(&harness.pool)
            .await
            .expect("shopping item");
    }
    add_text(&harness.pool, household, "Paper towels")
        .await
        .expect("manual item");
    add_text(&harness.pool, household, "Apples")
        .await
        .expect("manual item");
    sqlx::query("UPDATE shopping_items SET checked = 1 WHERE text = 'Apples'")
        .execute(&harness.pool)
        .await
        .expect("checked");
    let body = body_text(harness.get("/shopping", Some(&token)).await).await;
    let produce_at = body.find("Produce").expect("produce group");
    let dairy_at = body.find("Dairy").expect("dairy group");
    let other_at = body.find("Other").expect("other group");
    assert!(produce_at < dairy_at && dairy_at < other_at, "{body}");
    let unchecked_at = body.find("Cheese").expect("cheese");
    let checked_at = body.find("Apples").expect("checked item");
    assert!(unchecked_at < checked_at, "{body}");
    assert!(body.contains("action=\"/shopping/items/"), "{body}");
    assert!(body.contains("name=\"on\" value=\"false\""), "{body}");
    assert!(body.contains("name=\"on\" value=\"true\""), "{body}");
    assert!(body.contains("Paper towels"), "{body}");
    assert!(body.contains("action=\"/shopping/items\""), "{body}");
}

#[tokio::test]
async fn empty_list_and_quick_add_render() {
    let harness = Harness::start("shopping-page-empty").await;
    let (token, _) = list_context(&harness).await;
    let body = body_text(harness.get("/shopping", Some(&token)).await).await;
    assert!(body.contains("Your shopping list is empty."), "{body}");
    assert!(body.contains("action=\"/shopping/items\""), "{body}");
    assert_eq!(
        harness
            .post_form("/shopping/items", Some(&token), "text=Apples")
            .await
            .status(),
        StatusCode::SEE_OTHER
    );
    let body = body_text(harness.get("/shopping", Some(&token)).await).await;
    assert!(body.contains("Apples"), "{body}");
}
