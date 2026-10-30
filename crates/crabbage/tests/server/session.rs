//! Sessions: logout deletes the session on the server, and expired sessions require a new login.

use crate::common::{Harness, SESSION_COOKIE, header, session_token, set_cookie};
use axum::http::StatusCode;
use axum::http::header::LOCATION;
use crabbage_store::{sessions, users};

#[tokio::test]
async fn logout_deletes_the_session_and_clears_the_cookie() {
    let harness = Harness::start("session-logout").await;
    let token = session_token(&harness.sign_in().await).expect("sign-in sets a session");
    assert_eq!(harness.session_count().await, 1);

    let request = axum::http::Request::builder()
        .method("POST")
        .uri("/auth/logout")
        .header("cookie", format!("crabbage_session={token}"))
        .body(axum::body::Body::empty())
        .expect("request builds");
    let response = harness.send(request).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(header(&response, LOCATION).as_deref(), Some("/"));
    let cookie = set_cookie(&response, SESSION_COOKIE).expect("logout clears the cookie");
    assert!(cookie.contains("Max-Age=0"), "{cookie}");
    assert_eq!(harness.session_count().await, 0);

    let response = harness.get("/", Some(&token)).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(header(&response, LOCATION).as_deref(), Some("/auth/login"));
}

#[tokio::test]
async fn expired_session_requires_a_new_login() {
    let harness = Harness::start("session-expired").await;
    let user = users::upsert_user(&harness.pool, "subject-2", "Grace")
        .await
        .expect("user is stored");
    sessions::create(&harness.pool, "expired-token", user, 1)
        .await
        .expect("session is stored");
    let response = harness.get("/", Some("expired-token")).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(header(&response, LOCATION).as_deref(), Some("/auth/login"));
}

#[tokio::test]
async fn unknown_cookie_requires_a_new_login() {
    let harness = Harness::start("session-unknown").await;
    let response = harness.get("/", Some("made-up-token")).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(header(&response, LOCATION).as_deref(), Some("/auth/login"));
}
