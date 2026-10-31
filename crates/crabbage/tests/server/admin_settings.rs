//! The admin AI settings page and server functions.

use axum::http::StatusCode;

use crate::common::{Harness, body_text, session_token};

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
        ("test_ai_connection", ""),
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
