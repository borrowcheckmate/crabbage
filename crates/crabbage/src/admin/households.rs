#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces server function bodies with stubs"
    )
)]

use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos::server;
use serde::{Deserialize, Serialize};

use crate::i18n::t;
use crate::notice::{error_notice, failure, message};
use crate::user::current_user;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct HouseholdRow {
    id: i64,
    group_name: String,
    name: Option<String>,
    label: String,
}

#[server(prefix = "/api", endpoint = "list_households")]
async fn list_households() -> Result<Vec<HouseholdRow>, ServerFnError> {
    let pool = crate::access::require_admin().await?;
    let prefix = crate::access::config()?.household_group_prefix.clone();
    let rows = crabbage_store::households::list(&pool)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    Ok(rows
        .into_iter()
        .map(|row| HouseholdRow {
            id: row.id,
            label: crate::household::household_label(row.name.as_deref(), &row.group_name, &prefix),
            group_name: row.group_name,
            name: row.name,
        })
        .collect())
}

#[server(prefix = "/api", endpoint = "rename_household")]
async fn rename_household(id: i64, name: String) -> Result<(), ServerFnError> {
    let pool = crate::access::require_admin().await?;
    let found = crabbage_store::households::rename(&pool, id, Some(&name))
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    crate::access::ensure_found(found)
}

type PageData = (
    Result<crate::user::SignedInUser, ServerFnError>,
    Option<Result<Vec<HouseholdRow>, ServerFnError>>,
);

#[component]
pub(crate) fn HouseholdsPage() -> impl IntoView {
    let rename = ServerAction::<RenameHousehold>::new();
    let data = Resource::new(move || rename.version().get(), |_| load_page());
    view! { <Suspense fallback=|| view! { <p class="state-notice" role="status">{t("admin.loading")}</p> }>
        {move || Suspend::new(async move { page(data.await, rename) })}
    </Suspense> }
}

async fn load_page() -> PageData {
    let user = current_user().await;
    let rows = if user.as_ref().is_ok_and(|user| user.is_admin) {
        Some(list_households().await)
    } else {
        None
    };
    (user, rows)
}

fn page((user, rows): PageData, rename: ServerAction<RenameHousehold>) -> AnyView {
    if !matches!(user, Ok(user) if user.is_admin) {
        return error_notice(t("admin.not_allowed"));
    }
    match rows.unwrap_or_else(|| Err(ServerFnError::new("Households unavailable"))) {
        Ok(rows) => view! { <section class="admin-page"><h1>{t("admin.households.heading")}</h1>
            {move || rename.value().with(|value| failure(value.as_ref()))}
            <div class="admin-list">{rows.into_iter().map(|row| view! { <HouseholdRowView row=row rename=rename/> }).collect_view()}</div>
        </section> }.into_any(),
        Err(error) => error_notice(&message(&error)),
    }
}

#[component]
fn HouseholdRowView(row: HouseholdRow, rename: ServerAction<RenameHousehold>) -> impl IntoView {
    let id = row.id;
    let label = row.label;
    let group_name = row.group_name;
    view! { <section class="admin-row"><div><strong class="admin-row-title">{label.clone()}</strong><small class="admin-setting-hint">{group_name}</small></div>
        <ActionForm action=rename>
            <input type="hidden" name="id" value=id/>
            <label>{t_with_household(&label)} <input name="name" value=row.name.unwrap_or_default()/></label>
            <button>{t("admin.households.save")}</button>
        </ActionForm>
    </section> }
}

fn t_with_household(label: &str) -> String {
    crate::i18n::t_with("admin.households.name", &[("household", label)])
}
