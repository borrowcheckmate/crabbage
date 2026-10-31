//! Creating, changing and deleting recipes. Changing and deleting are refused to users who may not
//! do them, with 403.

#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces each server function body with a stub that does not await"
    )
)]

use leptos::prelude::ServerFnError;
use leptos::server;

/// Creates a recipe from the JSON draft and returns its id. The signed-in user becomes its owner.
#[server(prefix = "/api", endpoint = "create_recipe")]
pub(crate) async fn create_recipe(draft: String) -> Result<i64, ServerFnError> {
    use crabbage_store::recipes::create;
    let (pool, user) = crate::access::signed_in().await?;
    let input = crate::recipes::draft::parse(&draft)?;
    create(&pool, user, &input, crate::auth::now())
        .await
        .map_err(|error| crate::access::store_failure(&error))
}

/// Replaces the recipe `id` with the JSON draft.
#[server(prefix = "/api", endpoint = "update_recipe")]
pub(crate) async fn update_recipe(id: i64, draft: String) -> Result<(), ServerFnError> {
    use crate::recipes::rights::{Change, permit};
    use crabbage_store::recipes::update;
    let (pool, user) = crate::access::signed_in().await?;
    permit(&pool, user, id, Change::Edit).await?;
    let input = crate::recipes::draft::parse(&draft)?;
    let found = update(&pool, id, &input)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    crate::access::ensure_found(found)
}

/// Deletes the recipe `id` with everything that belongs to it.
#[server(prefix = "/api", endpoint = "delete_recipe")]
pub(crate) async fn delete_recipe(id: i64) -> Result<(), ServerFnError> {
    use crate::recipes::rights::{Change, load, permit};
    use crabbage_store::recipes::delete;
    let (pool, user) = crate::access::signed_in().await?;
    permit(&pool, user, id, Change::Delete).await?;
    let image = load(&pool, user, id).await?.image;
    let found = delete(&pool, id)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    crate::access::ensure_found(found)?;
    if let Some(name) = image {
        drop(crate::images::remove(&crate::access::data_dir()?, &name).await);
    }
    Ok(())
}
