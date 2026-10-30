//! The recipe list: every recipe, for any signed-in user.

#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces each server function body with a stub that does not await"
    )
)]

use leptos::prelude::ServerFnError;
use leptos::server;

use super::types::RecipeSummaryRow;

/// Every recipe, ordered by title.
#[server(prefix = "/api", endpoint = "list_recipes")]
pub(crate) async fn list_recipes() -> Result<Vec<RecipeSummaryRow>, ServerFnError> {
    use crabbage_store::recipes::list;
    let (pool, _user) = crate::access::signed_in().await?;
    let rows = list(&pool)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    Ok(rows.into_iter().map(RecipeSummaryRow::from).collect())
}
