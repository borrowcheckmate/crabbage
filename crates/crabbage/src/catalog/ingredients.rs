//! Admin endpoints for ingredients: create, rename, delete, aliases, store section, review and
//! merge.

#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces each server function body with a stub that does not await"
    )
)]

use leptos::prelude::ServerFnError;
use leptos::server;

use super::IngredientRow;

#[cfg(feature = "ssr")]
fn section_choice(value: &str) -> Result<Option<i64>, ServerFnError> {
    if value.is_empty() {
        return Ok(None);
    }
    value
        .parse()
        .ok()
        .map(Some)
        .ok_or_else(|| ServerFnError::new(crate::i18n::t("catalog.error.section")))
}

#[server(prefix = "/api", endpoint = "list_ingredients")]
pub(crate) async fn list_ingredients() -> Result<Vec<IngredientRow>, ServerFnError> {
    use crabbage_store::catalog::ingredients::list;
    let pool = crate::access::require_admin().await?;
    let rows = list(&pool)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    Ok(rows.into_iter().map(IngredientRow::from).collect())
}

#[server(prefix = "/api", endpoint = "list_pending_ingredients")]
pub(crate) async fn list_pending_ingredients() -> Result<Vec<IngredientRow>, ServerFnError> {
    use crabbage_store::catalog::ingredients::list_pending;
    let pool = crate::access::require_admin().await?;
    let rows = list_pending(&pool)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    Ok(rows.into_iter().map(IngredientRow::from).collect())
}

#[server(prefix = "/api", endpoint = "create_ingredient")]
pub(crate) async fn create_ingredient(name: String) -> Result<(), ServerFnError> {
    use crabbage_store::catalog::ingredients::create;
    let pool = crate::access::require_admin().await?;
    let name = crate::access::clean(&name)?;
    create(&pool, name)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    Ok(())
}

#[server(prefix = "/api", endpoint = "rename_ingredient")]
pub(crate) async fn rename_ingredient(id: i64, name: String) -> Result<(), ServerFnError> {
    use crabbage_store::catalog::ingredients::rename;
    let pool = crate::access::require_admin().await?;
    let name = crate::access::clean(&name)?;
    let found = rename(&pool, id, name)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    crate::access::ensure_found(found)
}

#[server(prefix = "/api", endpoint = "delete_ingredient")]
pub(crate) async fn delete_ingredient(id: i64) -> Result<(), ServerFnError> {
    use crabbage_store::catalog::ingredients::delete;
    let pool = crate::access::require_admin().await?;
    let found = delete(&pool, id)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    crate::access::ensure_found(found)
}

#[server(prefix = "/api", endpoint = "add_alias")]
pub(crate) async fn add_alias(ingredient_id: i64, alias: String) -> Result<(), ServerFnError> {
    use crabbage_store::catalog::ingredients::add_alias;
    let pool = crate::access::require_admin().await?;
    let alias = crate::access::clean(&alias)?;
    let found = add_alias(&pool, ingredient_id, alias)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    crate::access::ensure_found(found)
}

#[server(prefix = "/api", endpoint = "remove_alias")]
pub(crate) async fn remove_alias(ingredient_id: i64, alias: String) -> Result<(), ServerFnError> {
    use crabbage_store::catalog::ingredients::remove_alias;
    let pool = crate::access::require_admin().await?;
    let found = remove_alias(&pool, ingredient_id, &alias)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    crate::access::ensure_found(found)
}

/// The store section `section_id` names, where an empty value means none.
#[server(prefix = "/api", endpoint = "set_ingredient_section")]
pub(crate) async fn set_ingredient_section(
    id: i64,
    section_id: String,
) -> Result<(), ServerFnError> {
    use crabbage_store::catalog::ingredients::set_section;
    let pool = crate::access::require_admin().await?;
    let section = section_choice(&section_id)?;
    let found = set_section(&pool, id, section)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    crate::access::ensure_found(found)
}

#[server(prefix = "/api", endpoint = "approve_ingredient")]
pub(crate) async fn approve_ingredient(id: i64) -> Result<(), ServerFnError> {
    use crabbage_store::catalog::ingredients::approve;
    let pool = crate::access::require_admin().await?;
    let found = approve(&pool, id)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    crate::access::ensure_found(found)
}

#[server(prefix = "/api", endpoint = "merge_ingredients")]
pub(crate) async fn merge_ingredients(from: i64, into: i64) -> Result<(), ServerFnError> {
    use crabbage_store::catalog::ingredients::merge;
    let pool = crate::access::require_admin().await?;
    let found = merge(&pool, from, into)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    crate::access::ensure_found(found)
}
