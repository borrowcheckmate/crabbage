//! The users who may edit a recipe besides its owner and the admins.

use sqlx::sqlite::SqlitePool;

use crate::StoreError;

/// Lets user `user_id` edit recipe `recipe_id`. Adding an existing editor changes nothing. Returns
/// `false` when the recipe or the user does not exist.
///
/// # Errors
///
/// Returns an error if a query fails.
pub async fn add_editor(
    pool: &SqlitePool,
    recipe_id: i64,
    user_id: i64,
) -> Result<bool, StoreError> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM recipes WHERE id = ?1) \
         AND EXISTS (SELECT 1 FROM users WHERE id = ?2)",
    )
    .bind(recipe_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    if !exists {
        return Ok(false);
    }
    sqlx::query("INSERT OR IGNORE INTO recipe_editors (recipe_id, user_id) VALUES (?, ?)")
        .bind(recipe_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(true)
}

/// Removes user `user_id` from the editors of recipe `recipe_id`. Returns `false` when they were
/// not an editor.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn remove_editor(
    pool: &SqlitePool,
    recipe_id: i64,
    user_id: i64,
) -> Result<bool, StoreError> {
    let result = sqlx::query("DELETE FROM recipe_editors WHERE recipe_id = ? AND user_id = ?")
        .bind(recipe_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
