//! Admin endpoints for recipe categories: list, create, rename and delete.

#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces each server function body with a stub that does not await"
    )
)]

use leptos::prelude::ServerFnError;
use leptos::server;

use super::CategoryRow;

/// Every category, ordered by name.
#[server(prefix = "/api", endpoint = "list_categories")]
pub(crate) async fn list_categories() -> Result<Vec<CategoryRow>, ServerFnError> {
    use crabbage_store::catalog::categories::list;
    let pool = crate::access::require_admin().await?;
    let rows = list(&pool)
        .await
        .map_err(|error| category_failure(&error))?;
    Ok(rows.into_iter().map(CategoryRow::from).collect())
}

/// Creates a category named `name`.
#[server(prefix = "/api", endpoint = "create_category")]
pub(crate) async fn create_category(name: String) -> Result<(), ServerFnError> {
    use crabbage_store::catalog::categories::create;
    let pool = crate::access::require_admin().await?;
    let name = crate::access::clean(&name)?;
    create(&pool, name)
        .await
        .map_err(|error| category_failure(&error))?;
    Ok(())
}

/// Renames the category `id`.
#[server(prefix = "/api", endpoint = "rename_category")]
pub(crate) async fn rename_category(id: i64, name: String) -> Result<(), ServerFnError> {
    use crabbage_store::catalog::categories::rename;
    let pool = crate::access::require_admin().await?;
    let name = crate::access::clean(&name)?;
    let found = rename(&pool, id, name)
        .await
        .map_err(|error| category_failure(&error))?;
    crate::access::ensure_found(found)
}

/// Deletes the category `id` and takes it off every recipe.
#[server(prefix = "/api", endpoint = "delete_category")]
pub(crate) async fn delete_category(id: i64) -> Result<(), ServerFnError> {
    use crabbage_store::catalog::categories::delete;
    let pool = crate::access::require_admin().await?;
    let found = delete(&pool, id)
        .await
        .map_err(|error| category_failure(&error))?;
    crate::access::ensure_found(found)
}

/// The error for a failed category write: a taken name says so for categories.
#[cfg(feature = "ssr")]
fn category_failure(error: &crabbage_store::StoreError) -> ServerFnError {
    if matches!(error, crabbage_store::StoreError::Conflict(_)) {
        return ServerFnError::new(crate::i18n::t("admin.categories.taken"));
    }
    crate::access::store_failure(error)
}
