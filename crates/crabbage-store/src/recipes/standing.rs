//! Each user's rating of a recipe, from one to five stars, and their favourites. A user has at most
//! one rating per recipe.

use sqlx::{SqliteConnection, sqlite::SqlitePool};

use crate::StoreError;

/// Sets user `user_id`'s rating of recipe `recipe_id` to `stars`, replacing an earlier rating.
/// Returns `false`, and stores nothing, when the recipe does not exist.
///
/// # Errors
///
/// Returns [`StoreError::InvalidStars`] when `stars` is not from 1 to 5, before the database is
/// touched, or a database error.
pub async fn rate(
    pool: &SqlitePool,
    recipe_id: i64,
    user_id: i64,
    stars: i64,
) -> Result<bool, StoreError> {
    if !(1..=5).contains(&stars) {
        return Err(StoreError::InvalidStars);
    }
    let mut tx = crate::begin_write(pool).await?;
    if !recipe_exists(&mut tx, recipe_id).await? {
        return Ok(false);
    }
    sqlx::query(
        "INSERT INTO ratings (recipe_id, user_id, stars) VALUES (?, ?, ?) \
         ON CONFLICT (recipe_id, user_id) DO UPDATE SET stars = excluded.stars",
    )
    .bind(recipe_id)
    .bind(user_id)
    .bind(stars)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(true)
}

/// Makes recipe `recipe_id` a favourite of user `user_id` when `favourite` is true, or removes it
/// from their favourites when false. Returns `false`, and changes nothing, when the recipe does not
/// exist.
///
/// # Errors
///
/// Returns an error if a query fails.
pub async fn set_favourite(
    pool: &SqlitePool,
    recipe_id: i64,
    user_id: i64,
    favourite: bool,
) -> Result<bool, StoreError> {
    let mut tx = crate::begin_write(pool).await?;
    if !recipe_exists(&mut tx, recipe_id).await? {
        return Ok(false);
    }
    if favourite {
        sqlx::query("INSERT OR IGNORE INTO favourites (recipe_id, user_id) VALUES (?, ?)")
            .bind(recipe_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
    } else {
        sqlx::query("DELETE FROM favourites WHERE recipe_id = ? AND user_id = ?")
            .bind(recipe_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(true)
}

async fn recipe_exists(conn: &mut SqliteConnection, recipe_id: i64) -> Result<bool, StoreError> {
    let exists = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM recipes WHERE id = ?)")
        .bind(recipe_id)
        .fetch_one(conn)
        .await?;
    Ok(exists)
}
