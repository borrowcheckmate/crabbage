//! The admin AI settings page and server functions.

use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router, response::IntoResponse};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::common::{Harness, body_text, session_token};

type Requests = Arc<Mutex<Vec<(String, HeaderMap, Option<Value>)>>>;

async fn completion(
    State(requests): State<Requests>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    requests
        .lock()
        .await
        .push(("completion".to_owned(), headers, Some(body)));
    (StatusCode::UNAUTHORIZED, "saved-secret rejected")
}

async fn models(State(requests): State<Requests>, headers: HeaderMap) -> impl IntoResponse {
    requests
        .lock()
        .await
        .push(("models".to_owned(), headers, None));
    Json(serde_json::json!({"data": [{"id":"z-model"},{"id":"a-model"}]}))
}

async fn fake_provider() -> Result<(String, Requests), std::io::Error> {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let app = Router::new()
        .route("/v1/chat/completions", post(completion))
        .route("/v1/models", get(models))
        .with_state(Arc::clone(&requests));
    tokio::spawn(async move { drop(axum::serve(listener, app).await) });
    Ok((format!("http://{address}/v1"), requests))
}

fn form(pairs: &[(&str, &str)]) -> String {
    url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs)
        .finish()
}

#[tokio::test]
async fn every_settings_server_function_rejects_members() {
    let harness = Harness::start("admin-settings-access").await;
    let token = session_token(&harness.sign_in().await).expect("member session");
    for (endpoint, fields) in [
        ("admin_ai_settings", ""),
        (
            "save_ai_settings",
            "ai_enabled=false&provider=custom&base_url=https%3A%2F%2Fai.example%2Fv1&model=small&api_key=",
        ),
        ("remove_ai_key", ""),
        (
            "test_ai_connection",
            "provider=openai&base_url=https%3A%2F%2Fapi.openai.com%2Fv1&model=small&api_key=",
        ),
        (
            "list_ai_models",
            "provider=openai&base_url=https%3A%2F%2Fapi.openai.com%2Fv1&api_key=",
        ),
    ] {
        let response = harness
            .post_form(&format!("/api/{endpoint}"), Some(&token), fields)
            .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{endpoint}");
    }
}

#[tokio::test]
async fn settings_server_function_response_omits_the_saved_key() {
    let harness = Harness::start("admin-settings-response").await;
    let key = "server-function-secret-key";
    crabbage_store::settings::save(
        &harness.pool,
        &crabbage_store::settings::AiSettings {
            ai_enabled: false,
            provider: "custom".to_owned(),
            base_url: "https://ai.example/v1".to_owned(),
            model: "small".to_owned(),
            api_key: Some(key.to_owned()),
        },
    )
    .await
    .expect("settings saved");
    harness.issuer.set_groups(&["admin"]);
    let token = session_token(&harness.sign_in().await).expect("admin session");
    let page = body_text(harness.get("/admin/settings", Some(&token)).await).await;
    assert!(!page.contains(key), "{page}");
    assert!(page.contains("An API key is saved."), "{page}");
    assert!(page.contains("Remove saved key"), "{page}");
    let response = harness
        .post_form("/api/admin_ai_settings", Some(&token), "")
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("key_saved"), "{body}");
    assert!(!body.contains(key), "{body}");
}

#[tokio::test]
async fn settings_fields_are_stacked_and_test_connection_uses_one_button() {
    let harness = Harness::start("admin-settings-layout").await;
    harness.issuer.set_groups(&["admin"]);
    let token = session_token(&harness.sign_in().await).expect("admin session");
    let body = body_text(harness.get("/admin/settings", Some(&token)).await).await;
    let labels = [
        "Enable AI",
        "Provider",
        "Base URL",
        "Leave empty to use the provider default URL.",
        "Model",
        "API key",
        "No API key is saved.",
        "Save settings",
        "Test connection",
    ];
    let mut previous = 0;
    for label in labels {
        let position = body.find(label).expect("settings label is rendered");
        assert!(position >= previous, "{label} is out of order: {body}");
        previous = position;
    }
    assert_eq!(body.matches("Test connection").count(), 1, "{body}");
    assert!(
        body.contains("formaction=\"/api/test_ai_connection\""),
        "{body}"
    );
    assert!(body.contains("Load models"), "{body}");
    assert!(!body.contains("API key for this test"), "{body}");
    assert!(!body.contains("Remove saved key"), "{body}");
    assert!(body.contains("Back to administration"), "{body}");
}

#[tokio::test]
async fn administrators_can_update_and_remove_the_saved_key() {
    let harness = Harness::start("admin-settings-save").await;
    harness.issuer.set_groups(&["admin"]);
    let token = session_token(&harness.sign_in().await).expect("admin session");
    let response = harness
        .post_form(
            "/api/save_ai_settings",
            Some(&token),
            &form(&[
                ("ai_enabled", "true"),
                ("provider", "openai"),
                ("base_url", ""),
                ("model", "small"),
                ("api_key", "new-key"),
            ]),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let settings = crabbage_store::settings::get(&harness.pool)
        .await
        .expect("settings read");
    assert_eq!(settings.base_url, "https://api.openai.com/v1");
    assert_eq!(settings.api_key.as_deref(), Some("new-key"));
    let response = harness
        .post_form("/api/remove_ai_key", Some(&token), "")
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        crabbage_store::settings::get(&harness.pool)
            .await
            .expect("settings read")
            .api_key,
        None
    );
}

#[tokio::test]
async fn an_unticked_checkbox_saves_ai_as_disabled() {
    let harness = Harness::start("admin-settings-disabled").await;
    harness.issuer.set_groups(&["admin"]);
    let token = session_token(&harness.sign_in().await).expect("admin session");
    let response = harness
        .post_form(
            "/api/save_ai_settings",
            Some(&token),
            &form(&[
                ("provider", "mistral"),
                ("base_url", ""),
                ("model", "mistral-small-latest"),
                ("api_key", "new-key"),
            ]),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let settings = crabbage_store::settings::get(&harness.pool)
        .await
        .expect("settings read");
    assert!(!settings.ai_enabled);
    assert_eq!(settings.provider, "mistral");
}

#[tokio::test]
async fn test_and_model_listing_use_submitted_values_and_saved_key() {
    let (base, requests) = fake_provider().await.expect("fake provider starts");

    let harness = Harness::start("admin-settings-unsaved-test").await;
    crabbage_store::settings::save(
        &harness.pool,
        &crabbage_store::settings::AiSettings {
            ai_enabled: false,
            provider: "custom".to_owned(),
            base_url: "https://stored.example/v1".to_owned(),
            model: "stored-model".to_owned(),
            api_key: Some("saved-secret".to_owned()),
        },
    )
    .await
    .expect("saved settings");
    harness.issuer.set_groups(&["admin"]);
    let token = session_token(&harness.sign_in().await).expect("admin session");
    let tested = harness
        .post_form(
            "/api/test_ai_connection",
            Some(&token),
            &form(&[
                ("provider", "custom"),
                ("base_url", &base),
                ("model", "unsaved-model"),
                ("api_key", ""),
            ]),
        )
        .await;
    assert_eq!(tested.status(), StatusCode::OK);
    let tested = body_text(tested).await;
    assert!(!tested.contains("saved-secret"), "{tested}");

    let listed = harness
        .post_form(
            "/api/list_ai_models",
            Some(&token),
            &form(&[("provider", "custom"), ("base_url", &base), ("api_key", "")]),
        )
        .await;
    assert_eq!(listed.status(), StatusCode::OK);
    let listed = body_text(listed).await;
    assert!(
        listed.find("a-model").expect("first sorted model")
            < listed.find("z-model").expect("second sorted model"),
        "{listed}"
    );

    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    for (_, headers, _) in requests.iter() {
        assert_eq!(
            headers
                .get("authorization")
                .and_then(|value| value.to_str().ok()),
            Some("Bearer saved-secret")
        );
    }
    assert_eq!(
        requests[0].2.as_ref().expect("completion body")["model"],
        "unsaved-model"
    );
}
