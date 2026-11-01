#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces server function bodies with stubs"
    )
)]

#[cfg(feature = "ssr")]
use crate::i18n::t;
#[cfg(feature = "ssr")]
use crabbage_store::settings::{self, AiSettings};
use leptos::prelude::*;
use leptos::server;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PublicSettings {
    pub(crate) ai_enabled: bool,
    pub(crate) provider: String,
    pub(crate) base_url: String,
    pub(crate) model: String,
    pub(crate) key_saved: bool,
}

#[server(prefix = "/api", endpoint = "admin_ai_settings")]
pub(super) async fn load_settings() -> Result<PublicSettings, ServerFnError> {
    let pool = crate::access::require_admin().await?;
    let saved = settings::get(&pool).await.map_err(database_error)?;
    Ok(public(&saved))
}

#[server(prefix = "/api", endpoint = "save_ai_settings")]
pub(super) async fn save_settings(
    #[server(default)] ai_enabled: bool,
    provider: String,
    base_url: String,
    model: String,
    api_key: String,
) -> Result<(), ServerFnError> {
    let pool = crate::access::require_admin().await?;
    let base_url = resolve_url(&provider, &base_url)?;
    validate(ai_enabled, &base_url, &model)?;
    let key = (!api_key.trim().is_empty()).then(|| api_key.trim().to_owned());
    settings::save(
        &pool,
        &AiSettings {
            ai_enabled,
            provider,
            base_url,
            model: model.trim().to_owned(),
            api_key: key,
        },
    )
    .await
    .map_err(database_error)?;
    Ok(())
}

#[server(prefix = "/api", endpoint = "remove_ai_key")]
pub(super) async fn remove_saved_key() -> Result<(), ServerFnError> {
    let pool = crate::access::require_admin().await?;
    settings::remove_key(&pool).await.map_err(database_error)
}

#[server(prefix = "/api", endpoint = "test_ai_connection")]
pub(super) async fn test_connection(
    provider: String,
    base_url: String,
    model: String,
    api_key: String,
) -> Result<String, ServerFnError> {
    let pool = crate::access::require_admin().await?;
    let saved = settings::get(&pool).await.map_err(database_error)?;
    let base_url = resolve_url(&provider, &base_url)?;
    validate(true, &base_url, &model)?;
    let endpoint = client_url(&base_url)?;
    let key = (!api_key.trim().is_empty()).then(|| api_key.trim().to_owned());
    let key = key.as_deref().or(saved.api_key.as_deref());
    let (succeeded, output) = send_test(endpoint, model.trim(), key).await;
    Ok(if succeeded {
        output
    } else {
        format!("{} {output}", t("admin.settings.test_failed"))
    })
}

#[server(prefix = "/api", endpoint = "list_ai_models")]
pub(super) async fn list_ai_models(
    provider: String,
    base_url: String,
    api_key: String,
) -> Result<Vec<String>, ServerFnError> {
    let pool = crate::access::require_admin().await?;
    let saved = settings::get(&pool).await.map_err(database_error)?;
    let base_url = resolve_url(&provider, &base_url)?;
    let key = (!api_key.trim().is_empty()).then(|| api_key.trim().to_owned());
    let key = key.as_deref().or(saved.api_key.as_deref());
    list_models(models_url(&base_url)?, key).await
}

#[cfg(feature = "ssr")]
async fn send_test(endpoint: url::Url, model: &str, key: Option<&str>) -> (bool, String) {
    let Ok(http) = crate::auth::http_client() else {
        return (false, t("catalog.error.database").to_owned());
    };
    let body = serde_json::to_vec(&serde_json::json!({"model": model, "messages": [{"role":"user", "content":"Hi"}], "max_tokens": 1})).unwrap_or_default();
    let mut request = http
        .post(endpoint)
        .header("content-type", "application/json")
        .body(body);
    if let Some(key) = key {
        request = request.bearer_auth(key);
    }
    let result = request.send().await;
    let (succeeded, output) = match result {
        Ok(response) if response.status().is_success() => {
            (true, t("admin.settings.test_success").to_owned())
        }
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            (false, format!("HTTP {status}: {body}"))
        }
        Err(error) => (false, error.to_string()),
    };
    (succeeded, redact(&output, key))
}

#[cfg(feature = "ssr")]
async fn list_models(endpoint: url::Url, key: Option<&str>) -> Result<Vec<String>, ServerFnError> {
    let http = crate::auth::http_client()
        .map_err(|_error| ServerFnError::new(t("admin.settings.models_failed")))?;
    let mut request = http.get(endpoint);
    if let Some(key) = key {
        request = request.bearer_auth(key);
    }
    let response = request
        .send()
        .await
        .map_err(|error| ServerFnError::new(redact(&error.to_string(), key)))?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        let output = format!("HTTP {status}: {body}");
        return Err(ServerFnError::new(redact(&output, key)));
    }
    let body = response
        .text()
        .await
        .map_err(|error| ServerFnError::new(redact(&error.to_string(), key)))?;
    let value: serde_json::Value = serde_json::from_str(&body)
        .map_err(|error| ServerFnError::new(redact(&error.to_string(), key)))?;
    let mut ids = value
        .get("data")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            item.get("id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .collect::<Vec<_>>();
    ids.sort();
    Ok(ids)
}

#[cfg(feature = "ssr")]
fn redact(output: &str, key: Option<&str>) -> String {
    let redacted = key.map_or_else(
        || output.to_owned(),
        |secret| output.replace(secret, "[redacted]"),
    );
    redacted.chars().take(180).collect()
}

#[cfg(feature = "ssr")]
fn database_error(_: sqlx::Error) -> ServerFnError {
    ServerFnError::new(t("catalog.error.database"))
}

#[cfg(feature = "ssr")]
fn client_url(base: &str) -> Result<url::Url, ServerFnError> {
    endpoint_url(base, "/chat/completions")
}

#[cfg(feature = "ssr")]
fn models_url(base: &str) -> Result<url::Url, ServerFnError> {
    endpoint_url(base, "/models")
}

#[cfg(feature = "ssr")]
fn endpoint_url(base: &str, suffix: &str) -> Result<url::Url, ServerFnError> {
    let mut endpoint = url::Url::parse(base)
        .map_err(|_error| ServerFnError::new(t("admin.settings.invalid_url")))?;
    let mut path = endpoint.path().trim_end_matches('/').to_owned();
    path.push_str(suffix);
    endpoint.set_path(&path);
    endpoint.set_query(None);
    endpoint.set_fragment(None);
    Ok(endpoint)
}

#[cfg(feature = "ssr")]
fn resolve_url(provider: &str, base_url: &str) -> Result<String, ServerFnError> {
    if !matches!(
        provider,
        "openai" | "mistral" | "anthropic" | "openrouter" | "ollama" | "custom"
    ) {
        return Err(ServerFnError::new(t("admin.settings.invalid_url")));
    }
    let value = if base_url.trim().is_empty() {
        preset(provider).unwrap_or("")
    } else {
        base_url.trim()
    };
    let parsed = url::Url::parse(value)
        .map_err(|_error| ServerFnError::new(t("admin.settings.invalid_url")))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(ServerFnError::new(t("admin.settings.invalid_url")));
    }
    Ok(value.to_owned())
}

#[cfg(feature = "ssr")]
fn preset(provider: &str) -> Option<&'static str> {
    match provider {
        "openai" => Some("https://api.openai.com/v1"),
        "mistral" => Some("https://api.mistral.ai/v1"),
        "anthropic" => Some("https://api.anthropic.com/v1"),
        "openrouter" => Some("https://openrouter.ai/api/v1"),
        "ollama" => Some("http://localhost:11434/v1"),
        _ => None,
    }
}

#[cfg(feature = "ssr")]
fn validate(enabled: bool, url: &str, model: &str) -> Result<(), ServerFnError> {
    if url.trim().is_empty() {
        return Err(ServerFnError::new(t("admin.settings.invalid_url")));
    }
    if enabled && model.trim().is_empty() {
        return Err(ServerFnError::new(t("admin.settings.model_required")));
    }
    Ok(())
}

#[cfg(feature = "ssr")]
fn public(saved: &AiSettings) -> PublicSettings {
    PublicSettings {
        ai_enabled: saved.ai_enabled,
        provider: saved.provider.clone(),
        base_url: saved.base_url.clone(),
        model: saved.model.clone(),
        key_saved: saved.api_key.is_some(),
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::{preset, resolve_url};

    #[test]
    fn mistral_uses_its_openai_compatible_base_url() {
        assert_eq!(preset("mistral"), Some("https://api.mistral.ai/v1"));
        assert_eq!(
            resolve_url("mistral", "").as_deref(),
            Ok("https://api.mistral.ai/v1")
        );
    }
}
