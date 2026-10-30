//! Store sections: the ordered aisles an ingredient is shopped in.

use sqlx::sqlite::SqlitePool;

use super::names::unique_or_database;
use crate::StoreError;

/// A store section.
#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Section {
    /// The section's id.
    pub id: i64,
    /// The name, unique regardless of case.
    pub name: String,
    /// The place in the shopping order, from 0.
    pub position: i64,
}

/// Every store section, in shopping order.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn list(pool: &SqlitePool) -> Result<Vec<Section>, StoreError> {
    let sections =
        sqlx::query_as("SELECT id, name, position FROM store_sections ORDER BY position, id")
            .fetch_all(pool)
            .await?;
    Ok(sections)
}

/// Creates a section named `name` at the end of the order and returns its id.
///
/// # Errors
///
/// Returns [`StoreError::Conflict`] if the name is taken, or a database error.
pub async fn create(pool: &SqlitePool, name: &str) -> Result<i64, StoreError> {
    let id = sqlx::query_scalar(
        "INSERT INTO store_sections (name, position) \
         VALUES (?, (SELECT COALESCE(MAX(position) + 1, 0) FROM store_sections)) RETURNING id",
    )
    .bind(name)
    .fetch_one(pool)
    .await
    .map_err(unique_or_database(name))?;
    Ok(id)
}

/// Renames the section `id`. Returns `false` when there is no such section.
///
/// # Errors
///
/// Returns [`StoreError::Conflict`] if the name is taken, or a database error.
pub async fn rename(pool: &SqlitePool, id: i64, name: &str) -> Result<bool, StoreError> {
    let result = sqlx::query("UPDATE store_sections SET name = ? WHERE id = ?")
        .bind(name)
        .bind(id)
        .execute(pool)
        .await
        .map_err(unique_or_database(name))?;
    Ok(result.rows_affected() > 0)
}

/// Deletes the section `id`. Ingredients in it are left without a section. Returns `false` when
/// there is no such section.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, StoreError> {
    let result = sqlx::query("DELETE FROM store_sections WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Moves the section `id` to `position` and renumbers all sections 0 to n-1 in order. Positions
/// below 0 mean the start, positions past the end mean the end. Returns `false` when there is no
/// such section.
///
/// # Errors
///
/// Returns an error if a query fails. Nothing is changed in that case.
pub async fn move_section(pool: &SqlitePool, id: i64, position: i64) -> Result<bool, StoreError> {
    let mut tx = crate::begin_write(pool).await?;
    let mut ids: Vec<i64> =
        sqlx::query_scalar("SELECT id FROM store_sections ORDER BY position, id")
            .fetch_all(&mut *tx)
            .await?;
    if !ids.contains(&id) {
        return Ok(false);
    }
    ids.retain(|&other| other != id);
    let index = usize::try_from(position).unwrap_or(0).min(ids.len());
    ids.insert(index, id);
    for (new_position, section) in (0_i64..).zip(ids) {
        sqlx::query("UPDATE store_sections SET position = ? WHERE id = ?")
            .bind(new_position)
            .bind(section)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(true)
}
