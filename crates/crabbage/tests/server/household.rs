//! Households: group claims set the memberships on each login, and the user switches the active
//! one.

use axum::body::Body;
use axum::http::header::{CONTENT_TYPE, LOCATION};
use axum::http::{Request, StatusCode};
use crabbage_domain::membership::has_household_features;
use crabbage_store::households::{Household, active_household, households_of, sync_membership};
use crabbage_store::users::upsert_user;

use crate::common::{Harness, body_text, header, session_token};

const ADMIN: &str = "crabbage-admin";

async fn memberships(harness: &Harness, user: i64) -> Vec<String> {
    households_of(&harness.pool, user)
        .await
        .expect("households read")
        .into_iter()
        .map(|household| household.group_name)
        .collect()
}

async fn household_named(harness: &Harness, user: i64, group: &str) -> Household {
    households_of(&harness.pool, user)
        .await
        .expect("households read")
        .into_iter()
        .find(|household| household.group_name == group)
        .expect("the user is in the household")
}

async fn is_admin(harness: &Harness, user: i64) -> bool {
    sqlx::query_scalar("SELECT is_admin FROM users WHERE id = ?")
        .bind(user)
        .fetch_one(&harness.pool)
        .await
        .expect("user reads")
}

async fn post_household(
    harness: &Harness,
    token: &str,
    household_id: i64,
) -> axum::http::Response<Body> {
    let request = Request::builder()
        .method("POST")
        .uri("/household")
        .header("cookie", format!("crabbage_session={token}"))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!("household_id={household_id}")))
        .expect("request builds");
    harness.send(request).await
}

#[tokio::test]
async fn login_sets_the_admin_role_and_the_active_household_from_groups() {
    let harness = Harness::start("household-login").await;
    harness
        .issuer
        .set_groups(&[ADMIN, "crabbage-household-smith"]);
    let response = harness.sign_in().await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let user = harness.user_id().await;
    assert!(is_admin(&harness, user).await);
    let smith = household_named(&harness, user, "crabbage-household-smith").await;
    assert_eq!(
        memberships(&harness, user).await,
        ["crabbage-household-smith"]
    );
    assert_eq!(
        active_household(&harness.pool, user).await.unwrap(),
        Some(smith.id)
    );
}

#[tokio::test]
async fn changed_groups_take_effect_on_the_next_login() {
    let harness = Harness::start("household-next-login").await;
    harness
        .issuer
        .set_groups(&[ADMIN, "crabbage-household-smith"]);
    harness.sign_in().await;
    let user = harness.user_id().await;

    harness.issuer.set_groups(&["crabbage-household-jones"]);
    harness.sign_in().await;
    assert!(!is_admin(&harness, user).await);
    assert_eq!(
        memberships(&harness, user).await,
        ["crabbage-household-jones"]
    );
    let jones = household_named(&harness, user, "crabbage-household-jones").await;
    assert_eq!(
        active_household(&harness.pool, user).await.unwrap(),
        Some(jones.id)
    );
}

#[tokio::test]
async fn a_user_in_two_households_switches_and_the_choice_persists() {
    let harness = Harness::start("household-switch").await;
    harness
        .issuer
        .set_groups(&["crabbage-household-jones", "crabbage-household-smith"]);
    let token = session_token(&harness.sign_in().await).expect("sign-in sets a session");
    let user = harness.user_id().await;
    let smith = household_named(&harness, user, "crabbage-household-smith").await;
    assert_eq!(
        active_household(&harness.pool, user).await.unwrap(),
        Some(
            household_named(&harness, user, "crabbage-household-jones")
                .await
                .id
        )
    );

    let response = post_household(&harness, &token, smith.id).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(header(&response, LOCATION).as_deref(), Some("/"));
    assert_eq!(
        active_household(&harness.pool, user).await.unwrap(),
        Some(smith.id)
    );

    harness.sign_in().await;
    assert_eq!(
        active_household(&harness.pool, user).await.unwrap(),
        Some(smith.id)
    );
}

#[tokio::test]
async fn switching_to_a_household_the_user_is_not_in_is_forbidden() {
    let harness = Harness::start("household-forbidden").await;
    harness.issuer.set_groups(&["crabbage-household-smith"]);
    let token = session_token(&harness.sign_in().await).expect("sign-in sets a session");
    let user = harness.user_id().await;
    let smith = household_named(&harness, user, "crabbage-household-smith").await;

    let other = upsert_user(&harness.pool, "subject-2", "Other")
        .await
        .expect("user is stored");
    sync_membership(
        &harness.pool,
        other,
        false,
        &["crabbage-household-jones".to_owned()],
    )
    .await
    .expect("memberships are stored");
    let jones = household_named(&harness, other, "crabbage-household-jones").await;

    let response = post_household(&harness, &token, jones.id).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let response = post_household(&harness, &token, i64::MAX).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        active_household(&harness.pool, user).await.unwrap(),
        Some(smith.id)
    );
}

#[tokio::test]
async fn a_user_without_households_loads_recipes_without_household_features() {
    let harness = Harness::start("household-none").await;
    harness.issuer.set_groups(&["crabbage-household-"]);
    let token = session_token(&harness.sign_in().await).expect("sign-in sets a session");
    let user = harness.user_id().await;
    assert_eq!(memberships(&harness, user).await, Vec::<String>::new());
    let active = active_household(&harness.pool, user).await.unwrap();
    assert!(!has_household_features(active));

    let response = harness.get("/", Some(&token)).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        body_text(response)
            .await
            .contains("<title>crabbage</title>")
    );
}
