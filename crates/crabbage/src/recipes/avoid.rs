//! What the signed-in user's household avoids, for the recipe list and page.

use crabbage_store::planner::{self, Avoid};
use leptos::prelude::ServerFnError;
use sqlx::SqlitePool;

use crate::access::store_failure;

/// What the active household of `user` avoids; nothing without an active household.
pub(crate) async fn household_avoid(pool: &SqlitePool, user: i64) -> Result<Avoid, ServerFnError> {
    let household = crabbage_store::households::active_household(pool, user)
        .await
        .map_err(|error| store_failure(&error))?;
    match household {
        Some(household) => planner::avoid(pool, household)
            .await
            .map_err(|error| store_failure(&error)),
        None => Ok(Avoid::default()),
    }
}

/// The ids of the recipes that conflict with what the household of `user` avoids.
pub(crate) async fn avoided_recipes(
    pool: &SqlitePool,
    user: i64,
) -> Result<Vec<i64>, ServerFnError> {
    let avoid = household_avoid(pool, user).await?;
    planner::conflicting_recipes(pool, &avoid)
        .await
        .map_err(|error| store_failure(&error))
}
