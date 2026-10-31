use std::collections::HashSet;

use sqlx::sqlite::SqlitePool;

use super::model::RecipeInput;
use crate::StoreError;

/// Returns the Mealie ids already attached to recipes.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn imported_mealie_ids(pool: &SqlitePool) -> Result<HashSet<String>, StoreError> {
    let ids: Vec<String> =
        sqlx::query_scalar("SELECT mealie_id FROM recipes WHERE mealie_id IS NOT NULL")
            .fetch_all(pool)
            .await?;
    Ok(ids.into_iter().collect())
}

/// Creates a recipe with its source Mealie id in the same insert transaction.
///
/// # Errors
///
/// Returns an error if a step refers to a missing line or a database query fails.
pub async fn create_mealie(
    pool: &SqlitePool,
    owner_id: i64,
    input: &RecipeInput,
    now: i64,
    mealie_id: &str,
) -> Result<i64, StoreError> {
    super::write::create_inner(pool, owner_id, input, now, Some(mealie_id)).await
}
