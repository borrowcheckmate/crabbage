use super::server::{
    ListAiModels, PublicSettings, RemoveSavedKey, SaveSettings, TestConnection, load_settings,
};
use crate::i18n::t;
use crate::notice::{error_notice, message};
use crate::user::current_user;
use leptos::prelude::*;
use leptos::server::ServerAction;

#[component]
pub(crate) fn SettingsPage() -> impl IntoView {
    let save = ServerAction::<SaveSettings>::new();
    let remove = ServerAction::<RemoveSavedKey>::new();
    let test = ServerAction::<TestConnection>::new();
    let models = ServerAction::<ListAiModels>::new();
    let data = Resource::new(
        move || (save.version().get(), remove.version().get()),
        |_| load_page(),
    );
    view! { <Suspense fallback=|| view! { <p class="state-notice" role="status">{t("admin.loading")}</p> }>{move || Suspend::new(async move { page(data.await, save, remove, test, models) })}</Suspense> }
}

async fn load_page() -> (
    Result<crate::user::SignedInUser, ServerFnError>,
    Option<Result<PublicSettings, ServerFnError>>,
) {
    let user = current_user().await;
    let settings = if user.as_ref().is_ok_and(|user| user.is_admin) {
        Some(load_settings().await)
    } else {
        None
    };
    (user, settings)
}

fn page(
    (user, settings): (
        Result<crate::user::SignedInUser, ServerFnError>,
        Option<Result<PublicSettings, ServerFnError>>,
    ),
    save: ServerAction<SaveSettings>,
    remove: ServerAction<RemoveSavedKey>,
    test: ServerAction<TestConnection>,
    models: ServerAction<ListAiModels>,
) -> AnyView {
    if !matches!(user, Ok(user) if user.is_admin) {
        return error_notice(t("admin.not_allowed"));
    }
    let settings = match settings.unwrap_or_else(|| Err(ServerFnError::new("Settings unavailable")))
    {
        Ok(settings) => settings,
        Err(error) => return error_notice(&message(&error)),
    };
    let provider = settings.provider.clone();
    let base_url = settings.base_url.clone();
    let model = settings.model.clone();
    #[cfg(feature = "hydrate")]
    if settings.key_saved {
        let initial_provider = provider.clone();
        let initial_base_url = base_url.clone();
        Effect::new(move |_| {
            models.dispatch(ListAiModels {
                provider: initial_provider.clone(),
                base_url: initial_base_url.clone(),
                api_key: String::new(),
            });
        });
    }
    let provider_models = models;
    view! { <section class="admin-page admin-settings"><h1>{t("admin.settings.heading")}</h1><p><a href="/">{t("admin.settings.back")}</a></p>
        {move || save.value().with(|value| value.as_ref().and_then(|result| result.as_ref().err()).map(|error| error_notice(&message(error))))}
        {move || remove.value().with(|value| value.as_ref().and_then(|result| result.as_ref().err()).map(|error| error_notice(&message(error))))}
        <ActionForm action=save attr:class="admin-settings-form">
            <label class="admin-setting-toggle"><span>{t("admin.settings.enabled")}</span><input type="checkbox" name="ai_enabled" value="true" checked=settings.ai_enabled/></label>
            <label>{t("admin.settings.provider")}<select name="provider" on:change=move |event| provider_change(event, provider_models)><option value="openai" selected=provider == "openai">{t("admin.settings.openai")}</option><option value="mistral" selected=provider == "mistral">{t("admin.settings.mistral")}</option><option value="anthropic" selected=provider == "anthropic">{t("admin.settings.anthropic")}</option><option value="openrouter" selected=provider == "openrouter">{t("admin.settings.openrouter")}</option><option value="ollama" selected=provider == "ollama">{t("admin.settings.ollama")}</option><option value="custom" selected=provider == "custom">{t("admin.settings.custom")}</option></select></label>
            <label>{t("admin.settings.base_url")}<input name="base_url" type="url" value=base_url/><span class="admin-setting-hint">{t("admin.settings.base_url_hint")}</span></label>
            <label>{t("admin.settings.model")}<input name="model" list="ai-model-list" value=model/><button type="button" class="secondary" on:click=move |event| load_models(event, models)>{t("admin.settings.load_models")}</button><datalist id="ai-model-list">{move || models.value().with(|value| value.as_ref().and_then(|result| result.as_ref().ok()).map(|ids| ids.iter().map(|id| view! { <option value=id.clone()/> }).collect_view()))}</datalist>{move || models.value().with(|value| value.as_ref().and_then(|result| result.as_ref().err()).map(message)).map(|text| view! { <span role="status">{text}</span> })}</label>
            <label>{t("admin.settings.api_key")}<input name="api_key" type="password" autocomplete="new-password"/><span class="admin-setting-hint">{if settings.key_saved {t("admin.settings.key_saved")} else {t("admin.settings.key_missing")}}</span></label>
            <div class="admin-settings-actions"><button>{t("admin.settings.save")}</button><button class="secondary" formaction="/api/test_ai_connection" formmethod="post" formnovalidate on:click=move |event| test_current(event, test)>{t("admin.settings.test")}</button>{move || test.value().with(|value| value.as_ref().and_then(|result| result.as_ref().ok().cloned().or_else(|| result.as_ref().err().map(message)))).map(|text| view! { <span role="status">{text}</span> })}</div>
        </ActionForm>
        {settings.key_saved.then(|| view! { <ActionForm action=remove attr:class="admin-settings-action"><button class="danger">{t("admin.settings.remove_key")}</button></ActionForm> })}
    </section> }.into_any()
}

#[cfg(feature = "hydrate")]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos event handlers receive DOM events by value"
)]
fn provider_change(event: leptos::ev::Event, action: ServerAction<ListAiModels>) {
    use wasm_bindgen::JsCast;
    let Some(select) = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlSelectElement>().ok())
    else {
        return;
    };
    let Ok(Some(form)) = select.closest("form") else {
        return;
    };
    let Ok(Some(input)) = form.query_selector("input[name='base_url']") else {
        return;
    };
    let Ok(input) = input.dyn_into::<web_sys::HtmlInputElement>() else {
        return;
    };
    let preset = match select.value().as_str() {
        "openai" => "https://api.openai.com/v1",
        "mistral" => "https://api.mistral.ai/v1",
        "anthropic" => "https://api.anthropic.com/v1",
        "openrouter" => "https://openrouter.ai/api/v1",
        "ollama" => "http://localhost:11434/v1",
        _ => "",
    };
    if !preset.is_empty() {
        input.set_value(preset);
    }
    let base_url = input.value();
    let api_key = form
        .query_selector("input[name='api_key']")
        .ok()
        .flatten()
        .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default();
    action.dispatch(ListAiModels {
        provider: select.value(),
        base_url,
        api_key,
    });
}

#[cfg(not(feature = "hydrate"))]
fn provider_change(_: leptos::ev::Event, _: ServerAction<ListAiModels>) {}

#[cfg(feature = "hydrate")]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos event handlers receive DOM events by value"
)]
fn test_current(event: leptos::ev::MouseEvent, action: ServerAction<TestConnection>) {
    use wasm_bindgen::JsCast;
    event.prevent_default();
    let Some(button) = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlElement>().ok())
    else {
        return;
    };
    let Ok(Some(form)) = button.closest("form") else {
        return;
    };
    let get = |selector: &str| {
        form.query_selector(selector)
            .ok()
            .flatten()
            .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
            .map(|input| input.value())
            .unwrap_or_default()
    };
    let provider = form
        .query_selector("select[name='provider']")
        .ok()
        .flatten()
        .and_then(|element| element.dyn_into::<web_sys::HtmlSelectElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default();
    action.dispatch(TestConnection {
        provider,
        base_url: get("input[name='base_url']"),
        model: get("input[name='model']"),
        api_key: get("input[name='api_key']"),
    });
}

#[cfg(not(feature = "hydrate"))]
fn test_current(_: leptos::ev::MouseEvent, _: ServerAction<TestConnection>) {}

#[cfg(feature = "hydrate")]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos event handlers receive DOM events by value"
)]
fn load_models(event: leptos::ev::MouseEvent, action: ServerAction<ListAiModels>) {
    use wasm_bindgen::JsCast;
    event.prevent_default();
    let Some(button) = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlElement>().ok())
    else {
        return;
    };
    let Ok(Some(form)) = button.closest("form") else {
        return;
    };
    let get = |selector: &str| {
        form.query_selector(selector)
            .ok()
            .flatten()
            .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
            .map(|input| input.value())
            .unwrap_or_default()
    };
    let provider = form
        .query_selector("select[name='provider']")
        .ok()
        .flatten()
        .and_then(|element| element.dyn_into::<web_sys::HtmlSelectElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default();
    action.dispatch(ListAiModels {
        provider,
        base_url: get("input[name='base_url']"),
        api_key: get("input[name='api_key']"),
    });
}
#[cfg(not(feature = "hydrate"))]
fn load_models(_: leptos::ev::MouseEvent, _: ServerAction<ListAiModels>) {}
