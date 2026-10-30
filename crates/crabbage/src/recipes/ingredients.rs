//! The names of the known ingredients, for the ingredient field of a recipe. Any signed-in user may
//! read them; only admins change them.

#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces each server function body with a stub that does not await"
    )
)]

use leptos::prelude::ServerFnError;
use leptos::server;

/// The names of every ingredient, ordered by name.
#[server(prefix = "/api", endpoint = "ingredient_names")]
pub(crate) async fn ingredient_names() -> Result<Vec<String>, ServerFnError> {
    use crabbage_store::catalog::ingredients::list;
    let (pool, _user) = crate::access::signed_in().await?;
    let rows = list(&pool)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    Ok(rows.into_iter().map(|ingredient| ingredient.name).collect())
}
