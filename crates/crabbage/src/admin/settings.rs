#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces server function bodies with stubs"
    )
)]

#[cfg(feature = "ssr")]
use crabbage_store::settings::{self, AiSettings};
use leptos::prelude::*;
use leptos::server;

use crate::i18n::t;
use crate::notice::{error_notice, message};
use crate::user::current_user;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PublicSettings {
    pub(crate) ai_enabled: bool,
    pub(crate) provider: String,
    pub(crate) base_url: String,
    pub(crate) model: String,
    pub(crate) key_saved: bool,
}

#[server(prefix = "/api", endpoint = "admin_ai_settings")]
async fn load_settings() -> Result<PublicSettings, ServerFnError> {
    let pool = crate::access::require_admin().await?;
    let saved = settings::get(&pool).await.map_err(database_error)?;
    Ok(public(&saved))
}

#[server(prefix = "/api", endpoint = "save_ai_settings")]
async fn save_settings(
    ai_enabled: bool,
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
async fn remove_saved_key() -> Result<(), ServerFnError> {
    let pool = crate::access::require_admin().await?;
    settings::remove_key(&pool).await.map_err(database_error)
}

#[server(prefix = "/api", endpoint = "test_ai_connection")]
async fn test_connection() -> Result<String, ServerFnError> {
    let pool = crate::access::require_admin().await?;
    let saved = settings::get(&pool).await.map_err(database_error)?;
    let base_url = resolve_url(&saved.provider, &saved.base_url)?;
    validate(true, &base_url, &saved.model)?;
    let endpoint = client_url(&base_url)?;
    let (succeeded, output) =
        send_test(endpoint, saved.model.trim(), saved.api_key.as_deref()).await;
    Ok(if succeeded {
        output
    } else {
        format!("{} {output}", t("admin.settings.test_failed"))
    })
}

#[cfg(feature = "ssr")]
async fn send_test(endpoint: url::Url, model: &str, key: Option<&str>) -> (bool, String) {
    let http = openidconnect::reqwest::Client::new();
    let body = serde_json::to_vec(
        &serde_json::json!({"model": model, "messages": [{"role":"user", "content":"Hi"}], "max_tokens": 1}),
    );
    let Ok(body) = body else {
        return (false, t("catalog.error.database").to_owned());
    };
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
            let output = match response.text().await {
                Ok(body) => format!("HTTP {status}: {body}"),
                Err(_error) => format!("HTTP {status}"),
            };
            (false, output)
        }
        Err(error) => (false, error.to_string()),
    };
    let redacted = key.map_or_else(
        || output.clone(),
        |secret| output.replace(secret, "[redacted]"),
    );
    let short = redacted.chars().take(180).collect::<String>();
    (succeeded, short)
}

#[cfg(feature = "ssr")]
fn database_error(_: sqlx::Error) -> ServerFnError {
    ServerFnError::new(t("catalog.error.database"))
}

#[cfg(feature = "ssr")]
fn client_url(base: &str) -> Result<url::Url, ServerFnError> {
    let mut endpoint = url::Url::parse(base)
        .map_err(|_error| ServerFnError::new(t("admin.settings.invalid_url")))?;
    let mut path = endpoint.path().trim_end_matches('/').to_owned();
    path.push_str("/chat/completions");
    endpoint.set_path(&path);
    endpoint.set_query(None);
    endpoint.set_fragment(None);
    Ok(endpoint)
}

#[cfg(feature = "ssr")]
fn resolve_url(provider: &str, base_url: &str) -> Result<String, ServerFnError> {
    if !matches!(
        provider,
        "openai" | "anthropic" | "openrouter" | "ollama" | "custom"
    ) {
        return Err(ServerFnError::new(t("admin.settings.invalid_url")));
    }
    let entered = base_url.trim();
    let value = if entered.is_empty() {
        preset(provider).unwrap_or("")
    } else {
        entered
    };
    let parsed = url::Url::parse(value)
        .map_err(|_error| ServerFnError::new(t("admin.settings.invalid_url")))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(ServerFnError::new(t("admin.settings.invalid_url")));
    }
    Ok(value.to_owned())
}

fn preset(provider: &str) -> Option<&'static str> {
    match provider {
        "openai" => Some("https://api.openai.com/v1"),
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
    use std::sync::Arc;

    use axum::extract::State;
    use axum::http::HeaderMap;
    use axum::routing::post;
    use axum::{Json, Router};
    use serde_json::Value;
    use tokio::sync::Mutex;

    use super::{client_url, public, resolve_url, send_test, validate};
    use crabbage_store::settings::AiSettings;

    type Received = Arc<Mutex<Option<(HeaderMap, Value)>>>;

    #[test]
    fn settings_response_serialization_does_not_include_the_key() {
        let response = public(&AiSettings {
            ai_enabled: true,
            provider: "custom".to_owned(),
            base_url: "https://ai.example/v1".to_owned(),
            model: "small".to_owned(),
            api_key: Some("settings-response-secret".to_owned()),
        });
        let json = serde_json::to_string(&response).expect("response serializes");
        assert!(!json.contains("settings-response-secret"), "{json}");
        assert!(json.contains("\"key_saved\":true"), "{json}");
    }

    #[test]
    fn preset_urls_and_enabled_model_validation_are_enforced() {
        assert_eq!(
            resolve_url("anthropic", "").expect("preset URL"),
            "https://api.anthropic.com/v1"
        );
        assert!(resolve_url("custom", "ftp://ai.example/v1").is_err());
        assert!(resolve_url("other", "https://ai.example/v1").is_err());
        assert!(validate(true, "https://ai.example/v1", " ").is_err());
        assert!(validate(false, "https://ai.example/v1", " ").is_ok());
    }

    #[test]
    fn connection_endpoint_appends_the_completions_path_to_the_base() {
        assert_eq!(
            client_url("https://api.example/v1")
                .expect("endpoint")
                .as_str(),
            "https://api.example/v1/chat/completions"
        );
    }

    #[tokio::test]
    async fn connection_check_posts_minimal_request_and_redacts_provider_errors() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("fake provider binds");
        let address = listener.local_addr().expect("fake provider address");
        let received = Arc::new(Mutex::new(None));
        let app = Router::new()
            .route("/v1/chat/completions", post(fake_provider))
            .with_state(Arc::clone(&received));
        tokio::spawn(async move { drop(axum::serve(listener, app).await) });
        let endpoint = url::Url::parse(&format!("http://{address}/v1/"))
            .expect("fake provider URL")
            .join("chat/completions")
            .expect("endpoint URL");
        let (succeeded, error) = send_test(endpoint, "small", Some("provider-secret")).await;
        assert!(!succeeded);
        assert!(error.contains("[redacted]"), "{error}");
        assert!(!error.contains("provider-secret"), "{error}");
        let received = { received.lock().await.clone() };
        let (headers, body) = received.as_ref().expect("request recorded");
        assert_eq!(
            headers
                .get("authorization")
                .and_then(|value| value.to_str().ok()),
            Some("Bearer provider-secret")
        );
        assert_eq!(body["model"], "small");
        assert_eq!(body["max_tokens"], 1);
    }

    async fn fake_provider(
        State(received): State<Received>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> (axum::http::StatusCode, String) {
        *received.lock().await = Some((headers, body));
        (
            axum::http::StatusCode::UNAUTHORIZED,
            "provider-secret rejected".to_owned(),
        )
    }
}

#[component]
pub(crate) fn SettingsPage() -> impl IntoView {
    let save = ServerAction::<SaveSettings>::new();
    let remove = ServerAction::<RemoveSavedKey>::new();
    let test = ServerAction::<TestConnection>::new();
    view! { <Suspense fallback=|| view! { <p class="state-notice" role="status">{t("admin.loading")}</p> }>{move || Suspend::new(async move { page(save, remove, test).await })}</Suspense> }
}

async fn page(
    save: ServerAction<SaveSettings>,
    remove: ServerAction<RemoveSavedKey>,
    test: ServerAction<TestConnection>,
) -> AnyView {
    if !matches!(current_user().await, Ok(user) if user.is_admin) {
        return error_notice(t("admin.not_allowed"));
    }
    let settings = match load_settings().await {
        Ok(settings) => settings,
        Err(error) => return error_notice(&message(&error)),
    };
    let provider = settings.provider.clone();
    let base_url = settings.base_url.clone();
    let model = settings.model.clone();
    view! { <section class="admin-page admin-settings"><h1>{t("admin.settings.heading")}</h1>
        <p><a href="/">{t("admin.settings.back")}</a></p>
        {move || save.value().with(|value| value.as_ref().and_then(|result| result.as_ref().err()).map(|error| error_notice(&message(error))))}
        {move || remove.value().with(|value| value.as_ref().and_then(|result| result.as_ref().err()).map(|error| error_notice(&message(error))))}
        <ActionForm action=save attr:class="admin-settings-form">
            <label class="admin-setting-toggle"><span>{t("admin.settings.enabled")}</span><input type="checkbox" name="ai_enabled" value="true" checked=settings.ai_enabled/></label>
            <label>{t("admin.settings.provider")}<select name="provider" on:change=provider_change><option value="openai" selected=provider == "openai">{t("admin.settings.openai")}</option><option value="anthropic" selected=provider == "anthropic">{t("admin.settings.anthropic")}</option><option value="openrouter" selected=provider == "openrouter">{t("admin.settings.openrouter")}</option><option value="ollama" selected=provider == "ollama">{t("admin.settings.ollama")}</option><option value="custom" selected=provider == "custom">{t("admin.settings.custom")}</option></select></label>
            <label>{t("admin.settings.base_url")}<input name="base_url" type="url" value=base_url/><span class="admin-setting-hint">{t("admin.settings.base_url_hint")}</span></label>
            <label>{t("admin.settings.model")}<input name="model" value=model/></label>
            <label>{t("admin.settings.api_key")}<input name="api_key" type="password" autocomplete="new-password"/><span class="admin-setting-hint">{if settings.key_saved {t("admin.settings.key_saved")} else {t("admin.settings.key_missing")}}</span></label>
            <button>{t("admin.settings.save")}</button>
        </ActionForm>
        <ActionForm action=test attr:class="admin-settings-action"><button class="secondary">{t("admin.settings.test")}</button></ActionForm>
        {move || test.value().with(|value| value.as_ref().and_then(|result| result.as_ref().err()).map(|error| error_notice(&message(error))))}
        {move || test.value().with(|value| value.as_ref().and_then(|result| result.as_ref().ok()).map(|text| view! { <p class="notice" role="status">{text.clone()}</p> }))}
        {settings.key_saved.then(|| view! { <ActionForm action=remove attr:class="admin-settings-action"><button class="danger">{t("admin.settings.remove_key")}</button></ActionForm> })}
    </section> }.into_any()
}

#[cfg(feature = "hydrate")]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos event handlers receive DOM events by value"
)]
fn provider_change(event: leptos::ev::Event) {
    use wasm_bindgen::JsCast;
    let Some(select) = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlSelectElement>().ok())
    else {
        return;
    };
    let Some(form) = select.closest("form").ok().flatten() else {
        return;
    };
    let Some(input) = form.query_selector("input[name='base_url']").ok().flatten() else {
        return;
    };
    let Ok(input) = input.dyn_into::<web_sys::HtmlInputElement>() else {
        return;
    };
    if let Some(value) = preset(&select.value()) {
        input.set_value(value);
    }
}

#[cfg(not(feature = "hydrate"))]
fn provider_change(_: leptos::ev::Event) {}
