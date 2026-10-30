//! Recipe categories: the names recipes are filed under, listed by name.

use sqlx::sqlite::SqlitePool;

use super::names::unique_or_database;
use crate::StoreError;

/// A recipe category.
#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Category {
    /// The category's id.
    pub id: i64,
    /// The name, unique regardless of case.
    pub name: String,
}

/// Every category, ordered by name.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn list(pool: &SqlitePool) -> Result<Vec<Category>, StoreError> {
    let categories =
        sqlx::query_as("SELECT id, name FROM categories ORDER BY name COLLATE NOCASE, id")
            .fetch_all(pool)
            .await?;
    Ok(categories)
}

/// Creates a category named `name` and returns its id.
///
/// # Errors
///
/// Returns [`StoreError::Conflict`] if the name is taken, or a database error.
pub async fn create(pool: &SqlitePool, name: &str) -> Result<i64, StoreError> {
    let id = sqlx::query_scalar("INSERT INTO categories (name) VALUES (?) RETURNING id")
        .bind(name)
        .fetch_one(pool)
        .await
        .map_err(unique_or_database(name))?;
    Ok(id)
}

/// Renames the category `id`. Returns `false` when there is no such category.
///
/// # Errors
///
/// Returns [`StoreError::Conflict`] if the name is taken, or a database error.
pub async fn rename(pool: &SqlitePool, id: i64, name: &str) -> Result<bool, StoreError> {
    let result = sqlx::query("UPDATE categories SET name = ? WHERE id = ?")
        .bind(name)
        .bind(id)
        .execute(pool)
        .await
        .map_err(unique_or_database(name))?;
    Ok(result.rows_affected() > 0)
}

/// Deletes the category `id` and takes it off every recipe. Returns `false` when there is no such
/// category.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, StoreError> {
    let result = sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
