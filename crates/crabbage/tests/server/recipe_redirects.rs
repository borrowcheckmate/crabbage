use axum::http::StatusCode;

use crate::common::{Harness, sign_in_as};
use crate::recipes::{create, draft};

const OWNER: &str = "plain-form-owner";

#[tokio::test]
async fn a_plain_create_form_redirects_to_the_new_recipe() {
    let harness = Harness::start("recipes-plain-create").await;
    let token = sign_in_as(&harness, OWNER, &[]).await;
    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("draft", &draft("Pancakes"))
        .finish();
    let response = harness
        .post_form_html("/api/create_recipe", Some(&token), &body)
        .await;

    assert_eq!(response.status(), StatusCode::FOUND);
    let location = response
        .headers()
        .get("location")
        .expect("redirect location")
        .to_str()
        .expect("location is text");
    let id: i64 = location
        .strip_prefix("/recipes/")
        .expect("recipe path")
        .parse()
        .expect("recipe id");
    assert_eq!(location, format!("/recipes/{id}"));
}

#[tokio::test]
async fn a_plain_update_form_redirects_to_the_recipe() {
    let harness = Harness::start("recipes-plain-update").await;
    let token = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &token, "Pancakes").await;
    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("id", &id.to_string())
        .append_pair("draft", &draft("Crêpes"))
        .finish();
    let response = harness
        .post_form_html("/api/update_recipe", Some(&token), &body)
        .await;

    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(
        response
            .headers()
            .get("location")
            .expect("redirect location")
            .to_str()
            .expect("location is text"),
        format!("/recipes/{id}")
    );
}
