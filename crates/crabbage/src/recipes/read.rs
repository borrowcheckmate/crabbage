//! One recipe as its page shows it. Any signed-in user may view any recipe.

#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces each server function body with a stub that does not await"
    )
)]

use leptos::prelude::ServerFnError;
use leptos::server;

use super::types::RecipeDetail;

/// The recipe `id`, with the rights of the signed-in user to it.
#[server(prefix = "/api", endpoint = "get_recipe")]
pub(crate) async fn get_recipe(id: i64) -> Result<RecipeDetail, ServerFnError> {
    use crate::recipes::convert::detail;
    use crate::recipes::rights::{Rights, load};
    let (pool, user) = crate::access::signed_in().await?;
    let role = crate::access::role(&pool, user).await?;
    let recipe = load(&pool, user, id).await?;
    let rights = Rights::of(role, user, &recipe);
    let avoid = crate::recipes::avoid::household_avoid(&pool, user).await?;
    let mut detail = detail(recipe, rights);
    detail.avoided = avoid
        .codes()
        .filter(|code| {
            detail
                .allergens
                .iter()
                .chain(&detail.warnings)
                .any(|own| own == code)
        })
        .map(str::to_owned)
        .collect();
    Ok(detail)
}
