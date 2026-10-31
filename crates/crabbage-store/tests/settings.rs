//! AI settings persistence.

#![expect(
    clippy::expect_used,
    reason = "test setup and assertions stop on unexpected failures"
)]

mod common;

use crabbage_store::settings::{AiSettings, get, remove_key, save};

#[tokio::test]
async fn settings_start_disabled_save_and_retain_the_existing_key() {
    let (_dir, pool) = common::database("settings").await;
    let initial = get(&pool).await.expect("settings read");
    assert!(!initial.ai_enabled);
    assert!(!initial.is_enabled());
    assert_eq!(initial.provider, "openai");
    assert_eq!(initial.api_key, None);
    save(
        &pool,
        &AiSettings {
            ai_enabled: true,
            provider: "custom".to_owned(),
            base_url: "https://ai.example/v1".to_owned(),
            model: "small".to_owned(),
            api_key: Some("secret-key".to_owned()),
        },
    )
    .await
    .expect("settings saved");
    assert!(get(&pool).await.expect("settings read").is_enabled());
    save(
        &pool,
        &AiSettings {
            ai_enabled: false,
            provider: "ollama".to_owned(),
            base_url: "http://localhost:11434/v1".to_owned(),
            model: "local".to_owned(),
            api_key: None,
        },
    )
    .await
    .expect("settings updated");
    let updated = get(&pool).await.expect("settings read");
    assert!(!updated.ai_enabled);
    assert_eq!(updated.provider, "ollama");
    assert_eq!(updated.model, "local");
    assert_eq!(updated.api_key.as_deref(), Some("secret-key"));
    let debug = format!("{updated:?}");
    assert!(debug.contains("<redacted>"), "{debug}");
    assert!(!debug.contains("secret-key"), "{debug}");
    remove_key(&pool).await.expect("key removed");
    assert_eq!(get(&pool).await.expect("settings read").api_key, None);
}
