//! The categories a recipe can be filed under, for the filter and the form. Any signed-in user may
//! read them.

#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces each server function body with a stub that does not await"
    )
)]

use leptos::prelude::ServerFnError;
use leptos::server;

use crate::catalog::CategoryRow;

/// Every category, ordered by name.
#[server(prefix = "/api", endpoint = "list_recipe_categories")]
pub(crate) async fn list_recipe_categories() -> Result<Vec<CategoryRow>, ServerFnError> {
    use crabbage_store::catalog::categories::list;
    let (pool, _user) = crate::access::signed_in().await?;
    let rows = list(&pool)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    Ok(rows.into_iter().map(CategoryRow::from).collect())
}
