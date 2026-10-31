use std::time::Duration;

use axum::body::Body;
use axum::http::StatusCode;
use axum::http::header::LOCATION;
use crabbage_store::households::households_of;
use crabbage_store::shopping::add_text;
use http_body_util::BodyExt;

use crate::common::{Harness, header, sign_in_as};

const HOME: &str = "crabbage-household-home";
const OTHER_HOME: &str = "crabbage-household-away";

async fn next_frame(body: &mut Body) -> String {
    let frame = tokio::time::timeout(Duration::from_secs(2), body.frame())
        .await
        .expect("SSE frame arrives")
        .expect("SSE body remains open")
        .expect("SSE frame is valid");
    String::from_utf8(frame.into_data().expect("SSE frame is data").to_vec())
        .expect("SSE frame is UTF-8")
}

async fn open_stream(harness: &Harness, token: &str) -> Body {
    let response = harness.get("/shopping/events", Some(token)).await;
    assert_eq!(response.status(), StatusCode::OK);
    response.into_body()
}

async fn user_id(harness: &Harness, subject: &str) -> i64 {
    sqlx::query_scalar("SELECT id FROM users WHERE subject = ?")
        .bind(subject)
        .fetch_one(&harness.pool)
        .await
        .expect("user exists")
}

#[tokio::test]
async fn changes_reach_members_of_the_same_household_only() {
    let harness = Harness::start("shopping-stream-households").await;
    let first = sign_in_as(&harness, "first", &[HOME]).await;
    let second = sign_in_as(&harness, "second", &[HOME]).await;
    let away = sign_in_as(&harness, "away", &[OTHER_HOME]).await;
    let mut same_stream = open_stream(&harness, &second).await;
    let mut away_stream = open_stream(&harness, &away).await;
    let initial = next_frame(&mut same_stream).await;
    assert!(initial.contains("event: list"), "{initial}");
    let initial = next_frame(&mut away_stream).await;
    assert!(initial.contains("event: list"), "{initial}");

    assert_eq!(
        harness
            .post_form("/shopping/items", Some(&first), "text=Apples")
            .await
            .status(),
        StatusCode::SEE_OTHER
    );
    let update = next_frame(&mut same_stream).await;
    assert!(update.contains("Apples"), "{update}");
    assert!(
        tokio::time::timeout(Duration::from_millis(100), away_stream.frame())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn a_new_stream_starts_with_the_current_list() {
    let harness = Harness::start("shopping-stream-resync").await;
    let token = sign_in_as(&harness, "member", &[HOME]).await;
    let user = user_id(&harness, "member").await;
    let household = households_of(&harness.pool, user)
        .await
        .expect("households read")
        .first()
        .expect("household exists")
        .id;
    add_text(&harness.pool, household, "Added while away")
        .await
        .expect("item added");

    let mut stream = open_stream(&harness, &token).await;
    let initial = next_frame(&mut stream).await;
    assert!(initial.contains("Added while away"), "{initial}");
    let data = initial
        .split_once("data: ")
        .map(|(_, value)| value.trim())
        .expect("list data exists");
    let items: Vec<serde_json::Value> = serde_json::from_str(data).expect("list is JSON");
    assert_eq!(items.len(), 1);
}

#[tokio::test]
async fn shopping_forms_validate_values_and_household_ownership() {
    let harness = Harness::start("shopping-forms").await;
    let member = sign_in_as(&harness, "member", &[HOME]).await;
    let away = sign_in_as(&harness, "away", &[OTHER_HOME]).await;
    let user = user_id(&harness, "member").await;
    let household = households_of(&harness.pool, user)
        .await
        .expect("households read")
        .first()
        .expect("household exists")
        .id;
    let item = add_text(&harness.pool, household, "Private item")
        .await
        .expect("item added");

    assert_eq!(
        harness
            .post_form("/shopping/items", Some(&member), "text=%20%20")
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        harness
            .post_form(
                "/shopping/checked",
                Some(&member),
                &format!("ids={item}&on=yes")
            )
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        harness
            .post_form(
                "/shopping/checked",
                Some(&away),
                &format!("ids={item}&on=true")
            )
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        harness
            .post_form(&format!("/shopping/items/{item}/delete"), Some(&away), "",)
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
    let success = harness
        .post_form("/shopping/items", Some(&member), "text=Milk")
        .await;
    assert_eq!(success.status(), StatusCode::SEE_OTHER);
    assert_eq!(header(&success, LOCATION).as_deref(), Some("/shopping"));
}

#[tokio::test]
async fn the_event_stream_requires_a_valid_session() {
    let harness = Harness::start("shopping-stream-auth").await;
    let response = harness.get("/shopping/events", None).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(header(&response, LOCATION).as_deref(), Some("/auth/login"));
}
