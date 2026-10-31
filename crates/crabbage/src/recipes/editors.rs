//! Editors of a recipe, and the users to pick them from. Only the owner and admins change editors.

#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces each server function body with a stub that does not await"
    )
)]

use leptos::prelude::ServerFnError;
use leptos::server;

use super::types::PersonRow;

/// Every user, by name, to pick an editor from.
#[server(prefix = "/api", endpoint = "list_users")]
pub(crate) async fn list_users() -> Result<Vec<PersonRow>, ServerFnError> {
    let (pool, _user) = crate::access::signed_in().await?;
    let people = crabbage_store::recipes::list_users(&pool)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    Ok(people.into_iter().map(PersonRow::from).collect())
}

/// Lets user `user_id` edit recipe `recipe_id`. Returns the editors afterwards.
#[server(prefix = "/api", endpoint = "add_recipe_editor")]
pub(crate) async fn add_recipe_editor(
    recipe_id: i64,
    user_id: i64,
) -> Result<Vec<PersonRow>, ServerFnError> {
    use crate::recipes::rights::{Change, load, permit};
    use crabbage_store::recipes::add_editor;
    let (pool, user) = crate::access::signed_in().await?;
    permit(&pool, user, recipe_id, Change::ManageEditors).await?;
    let found = add_editor(&pool, recipe_id, user_id)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    crate::access::ensure_found(found)?;
    let recipe = load(&pool, user, recipe_id).await?;
    Ok(recipe.editors.into_iter().map(PersonRow::from).collect())
}

/// Removes user `user_id` from the editors of recipe `recipe_id`. Returns the editors afterwards.
#[server(prefix = "/api", endpoint = "remove_recipe_editor")]
pub(crate) async fn remove_recipe_editor(
    recipe_id: i64,
    user_id: i64,
) -> Result<Vec<PersonRow>, ServerFnError> {
    use crate::recipes::rights::{Change, load, permit};
    use crabbage_store::recipes::remove_editor;
    let (pool, user) = crate::access::signed_in().await?;
    permit(&pool, user, recipe_id, Change::ManageEditors).await?;
    let found = remove_editor(&pool, recipe_id, user_id)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    crate::access::ensure_found(found)?;
    let recipe = load(&pool, user, recipe_id).await?;
    Ok(recipe.editors.into_iter().map(PersonRow::from).collect())
}
