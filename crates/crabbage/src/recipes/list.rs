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

/// Every recipe ordered by title, as the signed-in user sees it. `category` keeps only the recipes in
/// that category, `meal_type` (a meal type code) only those of that meal type, `favourites_only` only the user's favourites and `search` only the recipes that
/// match it by title or ingredient.
#[server(prefix = "/api", endpoint = "list_recipes")]
pub(crate) async fn list_recipes(
    category: Option<i64>,
    meal_type: Option<String>,
    favourites_only: bool,
    search: Option<String>,
) -> Result<Vec<RecipeSummaryRow>, ServerFnError> {
    use crabbage_domain::meal_types::MealType;
    use crabbage_store::recipes::list;
    let (pool, user) = crate::access::signed_in().await?;
    let meal_type = meal_type.as_deref().and_then(MealType::from_code);
    let rows = list(
        &pool,
        category,
        meal_type,
        favourites_only,
        search.as_deref(),
        user,
    )
    .await
    .map_err(|error| crate::access::store_failure(&error))?;
    Ok(rows.into_iter().map(RecipeSummaryRow::from).collect())
}
