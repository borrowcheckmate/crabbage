//! Admin endpoints for store sections: create, rename, delete and reorder.

#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces each server function body with a stub that does not await"
    )
)]

use leptos::prelude::ServerFnError;
use leptos::server;

use super::SectionRow;

#[server(prefix = "/api", endpoint = "list_sections")]
pub(crate) async fn list_sections() -> Result<Vec<SectionRow>, ServerFnError> {
    use crabbage_store::catalog::sections::list;
    let pool = crate::access::require_admin().await?;
    let rows = list(&pool)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    Ok(rows.into_iter().map(SectionRow::from).collect())
}

#[server(prefix = "/api", endpoint = "create_section")]
pub(crate) async fn create_section(name: String) -> Result<(), ServerFnError> {
    use crabbage_store::catalog::sections::create;
    let pool = crate::access::require_admin().await?;
    let name = crate::access::clean(&name)?;
    create(&pool, name)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    Ok(())
}

#[server(prefix = "/api", endpoint = "rename_section")]
pub(crate) async fn rename_section(id: i64, name: String) -> Result<(), ServerFnError> {
    use crabbage_store::catalog::sections::rename;
    let pool = crate::access::require_admin().await?;
    let name = crate::access::clean(&name)?;
    let found = rename(&pool, id, name)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    crate::access::ensure_found(found)
}

#[server(prefix = "/api", endpoint = "delete_section")]
pub(crate) async fn delete_section(id: i64) -> Result<(), ServerFnError> {
    use crabbage_store::catalog::sections::delete;
    let pool = crate::access::require_admin().await?;
    let found = delete(&pool, id)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    crate::access::ensure_found(found)
}

#[server(prefix = "/api", endpoint = "move_section")]
pub(crate) async fn move_section(id: i64, position: i64) -> Result<(), ServerFnError> {
    use crabbage_store::catalog::sections::move_section;
    let pool = crate::access::require_admin().await?;
    let found = move_section(&pool, id, position)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    crate::access::ensure_found(found)
}
