//! The Mealie migration page: the start form, the progress while it runs and the summary after.

#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces the server function body with a stub that does not await"
    )
)]

use leptos::prelude::*;
use leptos::server;

use crate::i18n::{t, t_with};
use crate::notice::error_notice;
use crate::user::current_user;

/// The state of the latest migration run.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct RunState {
    /// Whether the run is still going.
    pub(crate) running: bool,
    /// The recipes handled so far.
    pub(crate) done: usize,
    /// The recipes Mealie listed.
    pub(crate) total: usize,
    /// The recipes imported in this run.
    pub(crate) imported: usize,
    /// The recipes skipped because an earlier run imported them.
    pub(crate) skipped: usize,
    /// The name of each recipe that failed, with the reason.
    pub(crate) failed: Vec<(String, String)>,
    /// The error that stopped the run before any recipe, if one did.
    pub(crate) error: Option<String>,
}

#[server(prefix = "/api", endpoint = "mealie_migration_status")]
async fn migration_status() -> Result<RunState, ServerFnError> {
    crate::access::require_admin().await?;
    let status = use_context::<std::sync::Arc<tokio::sync::Mutex<Option<RunState>>>>()
        .ok_or_else(|| ServerFnError::new(t("catalog.error.database")))?;
    let status = status.lock().await.clone();
    Ok(status.unwrap_or_default())
}

/// The Mealie migration page, for admins only.
#[component]
pub(crate) fn MealiePage() -> impl IntoView {
    view! {
        <Suspense fallback=|| ()>
            {move || Suspend::new(async move { page().await })}
        </Suspense>
    }
}

async fn page() -> AnyView {
    if !matches!(current_user().await, Ok(user) if user.is_admin) {
        return error_notice(t("admin.not_allowed"));
    }
    let status = migration_status().await.unwrap_or_default();
    view! {
        <h1>{t("admin.mealie.heading")}</h1>
        {status.running.then(|| view! { <meta http-equiv="refresh" content="2"/> })}
        <form method="post" action="/admin/mealie/start">
            <label>{t("admin.mealie.url")} <input name="url" type="url" required/></label>
            <label>{t("admin.mealie.token")} <input name="token" type="password" required autocomplete="off"/></label>
            <button disabled=status.running>{t("admin.mealie.start")}</button>
        </form>
        {status_view(status)}
    }.into_any()
}

fn status_view(status: RunState) -> AnyView {
    if let Some(error) = status.error {
        return error_notice(&error);
    }
    if status.total == 0 && status.failed.is_empty() && status.imported == 0 && status.skipped == 0
    {
        return ().into_any();
    }
    view! {
        <section class="card" aria-live="polite">
            {status.running.then(|| view! { <p>{t_with("admin.mealie.progress", &[("done", &status.done.to_string()), ("total", &status.total.to_string())])}</p> })}
            <p>{t_with("admin.mealie.imported", &[("count", &status.imported.to_string())])}</p>
            <p>{t_with("admin.mealie.skipped", &[("count", &status.skipped.to_string())])}</p>
            <p>{t_with("admin.mealie.failed", &[("count", &status.failed.len().to_string())])}</p>
            <ul>{status.failed.into_iter().map(|(name, reason)| view! { <li>{format!("{name}: {reason}")}</li> }).collect_view()}</ul>
        </section>
    }.into_any()
}
