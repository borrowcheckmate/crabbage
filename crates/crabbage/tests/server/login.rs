//! Sign-in: the redirect to the provider, the callback that creates a session and its cookie.

use axum::http::StatusCode;
use axum::http::header::LOCATION;
use url::Url;

use crate::common::{
    Harness, LOGIN_COOKIE, SESSION_COOKIE, authorization_params, body_text, cookie_pair, header,
    session_token, set_cookie,
};

#[tokio::test]
async fn unauthenticated_pages_redirect_to_login() {
    let harness = Harness::start("auth-redirect").await;
    let response = harness.get("/", None).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(header(&response, LOCATION).as_deref(), Some("/auth/login"));
}

#[tokio::test]
async fn health_check_and_sign_in_need_no_session() {
    let harness = Harness::start("auth-public").await;
    assert_eq!(harness.get("/healthz", None).await.status(), StatusCode::OK);
    assert_eq!(
        harness.get("/auth/login", None).await.status(),
        StatusCode::SEE_OTHER
    );
}

#[tokio::test]
async fn login_redirects_to_the_provider_with_pkce_state_and_nonce() {
    let harness = Harness::start("auth-login").await;
    let response = harness.get("/auth/login", None).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let location = header(&response, LOCATION).expect("redirect has a location");
    assert!(location.starts_with(harness.issuer.url()), "{location}");
    let url = Url::parse(&location).expect("location is a URL");
    assert_eq!(url.path(), "/authorize");
    let query = |name: &str| {
        url.query_pairs()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.into_owned())
    };
    assert_eq!(query("code_challenge_method").as_deref(), Some("S256"));
    assert!(query("code_challenge").is_some_and(|value| !value.is_empty()));
    assert!(query("state").is_some_and(|value| !value.is_empty()));
    assert!(query("nonce").is_some_and(|value| !value.is_empty()));
    assert_eq!(
        query("redirect_uri").as_deref(),
        Some("https://recipes.example.com/auth/callback")
    );
    assert_eq!(query("scope").as_deref(), Some("openid profile groups"));
}

#[tokio::test]
async fn login_binds_the_state_to_this_browser_with_a_short_lived_cookie() {
    let harness = Harness::start("auth-login-cookie").await;
    let response = harness.get("/auth/login", None).await;
    let (state, _nonce) = authorization_params(&response);
    let cookie = set_cookie(&response, LOGIN_COOKIE).expect("login sets the login cookie");
    assert_eq!(
        cookie_pair(&response, LOGIN_COOKIE),
        Some(format!("{LOGIN_COOKIE}={state}"))
    );
    for attribute in [
        "HttpOnly",
        "Secure",
        "SameSite=Lax",
        "Path=/auth/",
        "Max-Age=600",
    ] {
        assert!(cookie.contains(attribute), "{cookie}");
    }
}

#[tokio::test]
async fn callback_with_the_right_state_sets_a_secure_session_cookie() {
    let harness = Harness::start("auth-callback").await;
    let response = harness.sign_in().await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(header(&response, LOCATION).as_deref(), Some("/"));
    let cookie = set_cookie(&response, SESSION_COOKIE).expect("callback sets the session cookie");
    for attribute in [
        "HttpOnly",
        "Secure",
        "SameSite=Lax",
        "Path=/",
        "Max-Age=2592000",
    ] {
        assert!(cookie.contains(attribute), "{cookie}");
    }
    let clear = set_cookie(&response, LOGIN_COOKIE).expect("callback clears the login cookie");
    assert!(clear.contains("Max-Age=0"), "{clear}");
    let token = session_token(&response).expect("cookie carries a token");
    assert_eq!(
        harness.get("/", Some(&token)).await.status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn callback_with_an_unknown_state_is_rejected() {
    let harness = Harness::start("auth-unknown-state").await;
    let response = harness
        .get_with_cookies(
            "/auth/callback?code=code&state=never-issued",
            "crabbage_login=never-issued",
        )
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(set_cookie(&response, SESSION_COOKIE).is_none());
}

#[tokio::test]
async fn callback_without_the_login_cookie_is_rejected_and_keeps_the_attempt() {
    let harness = Harness::start("auth-no-cookie").await;
    let login = harness.get("/auth/login", None).await;
    let (state, nonce) = authorization_params(&login);
    let login_cookie = cookie_pair(&login, LOGIN_COOKIE).expect("login sets the login cookie");
    harness.issuer.expect_nonce(&nonce);
    let uri = format!("/auth/callback?code=code&state={state}");

    let response = harness.get(&uri, None).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(harness.session_count().await, 0);

    let response = harness.get_with_cookies(&uri, &login_cookie).await;
    assert_eq!(
        response.status(),
        StatusCode::SEE_OTHER,
        "the attempt was not consumed"
    );
}

#[tokio::test]
async fn login_cookie_for_another_state_is_rejected_and_keeps_the_attempt() {
    let harness = Harness::start("auth-other-cookie").await;
    let login = harness.get("/auth/login", None).await;
    let (state, nonce) = authorization_params(&login);
    let login_cookie = cookie_pair(&login, LOGIN_COOKIE).expect("login sets the login cookie");
    harness.issuer.expect_nonce(&nonce);
    let uri = format!("/auth/callback?code=code&state={state}");

    let response = harness
        .get_with_cookies(&uri, "crabbage_login=someone-elses-state")
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(harness.session_count().await, 0);

    let response = harness.get_with_cookies(&uri, &login_cookie).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn a_login_state_works_only_once() {
    let harness = Harness::start("auth-state-once").await;
    let login = harness.get("/auth/login", None).await;
    let (state, nonce) = authorization_params(&login);
    let login_cookie = cookie_pair(&login, LOGIN_COOKIE).expect("login sets the login cookie");
    harness.issuer.expect_nonce(&nonce);
    let uri = format!("/auth/callback?code=code&state={state}");
    assert_eq!(
        harness.get_with_cookies(&uri, &login_cookie).await.status(),
        StatusCode::SEE_OTHER
    );
    assert_eq!(
        harness.get_with_cookies(&uri, &login_cookie).await.status(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn wrong_nonce_in_the_id_token_creates_no_session() {
    let harness = Harness::start("auth-wrong-nonce").await;
    let login = harness.get("/auth/login", None).await;
    let (state, _nonce) = authorization_params(&login);
    let login_cookie = cookie_pair(&login, LOGIN_COOKIE).expect("login sets the login cookie");
    harness.issuer.expect_nonce("not-the-nonce-we-sent");
    let response = harness
        .get_with_cookies(
            &format!("/auth/callback?code=code&state={state}"),
            &login_cookie,
        )
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_text(response).await, "login failed");
    assert_eq!(harness.session_count().await, 0);
}
