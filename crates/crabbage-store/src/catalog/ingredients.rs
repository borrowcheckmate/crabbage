//! Ingredients, their aliases, store sections, the review queue and merging.

use sqlx::sqlite::SqlitePool;

use super::names::{ensure_not_a_name, ensure_not_an_alias, unique_or_database};
use crate::StoreError;

/// An ingredient with its aliases.
#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Ingredient {
    /// The ingredient's id.
    pub id: i64,
    /// The name, unique regardless of case.
    pub name: String,
    /// The store section the ingredient is shopped in, if one is chosen.
    pub store_section_id: Option<i64>,
    /// Whether the ingredient was added by a user and waits for an admin to approve it.
    pub pending_review: bool,
    /// The alternative names, ordered by alias.
    #[sqlx(skip)]
    pub aliases: Vec<String>,
}

/// Every ingredient, ordered by name.
///
/// # Errors
///
/// Returns an error if a query fails.
pub async fn list(pool: &SqlitePool) -> Result<Vec<Ingredient>, StoreError> {
    load(
        pool,
        "SELECT id, name, store_section_id, pending_review FROM ingredients ORDER BY name",
    )
    .await
}

/// The ingredients waiting for review, ordered by name.
///
/// # Errors
///
/// Returns an error if a query fails.
pub async fn list_pending(pool: &SqlitePool) -> Result<Vec<Ingredient>, StoreError> {
    load(
        pool,
        "SELECT id, name, store_section_id, pending_review FROM ingredients \
         WHERE pending_review = 1 ORDER BY name",
    )
    .await
}

async fn load(pool: &SqlitePool, sql: &'static str) -> Result<Vec<Ingredient>, StoreError> {
    let mut ingredients: Vec<Ingredient> = sqlx::query_as(sql).fetch_all(pool).await?;
    let aliases: Vec<(i64, String)> =
        sqlx::query_as("SELECT ingredient_id, alias FROM ingredient_aliases ORDER BY alias")
            .fetch_all(pool)
            .await?;
    for ingredient in &mut ingredients {
        ingredient.aliases = aliases
            .iter()
            .filter(|(owner, _)| *owner == ingredient.id)
            .map(|(_, alias)| alias.clone())
            .collect();
    }
    Ok(ingredients)
}

/// Creates an ingredient named `name`, approved, and returns its id.
///
/// # Errors
///
/// Returns [`StoreError::Conflict`] if `name` is already a name or alias, or a database error.
pub async fn create(pool: &SqlitePool, name: &str) -> Result<i64, StoreError> {
    let mut tx = crate::begin_write(pool).await?;
    ensure_not_an_alias(&mut tx, name).await?;
    let id = sqlx::query_scalar("INSERT INTO ingredients (name) VALUES (?) RETURNING id")
        .bind(name)
        .fetch_one(&mut *tx)
        .await
        .map_err(unique_or_database(name))?;
    tx.commit().await?;
    Ok(id)
}

/// Renames the ingredient `id` to `name`. Returns `false` when there is no such ingredient.
///
/// # Errors
///
/// Returns [`StoreError::Conflict`] if `name` is already a name or alias, or a database error.
pub async fn rename(pool: &SqlitePool, id: i64, name: &str) -> Result<bool, StoreError> {
    let mut tx = crate::begin_write(pool).await?;
    ensure_not_an_alias(&mut tx, name).await?;
    let result = sqlx::query("UPDATE ingredients SET name = ? WHERE id = ?")
        .bind(name)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(unique_or_database(name))?;
    tx.commit().await?;
    Ok(result.rows_affected() > 0)
}

/// Deletes the ingredient `id` and its aliases. Returns `false` when there is no such ingredient.
///
/// # Errors
///
/// Returns [`StoreError::InUse`] while recipe lines use the ingredient, or a database error.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, StoreError> {
    let in_use: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM recipe_lines WHERE ingredient_id = ?)")
            .bind(id)
            .fetch_one(pool)
            .await?;
    if in_use {
        return Err(StoreError::InUse);
    }
    let result = sqlx::query("DELETE FROM ingredients WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Adds `alias` to the ingredient `id`. Returns `false` when there is no such ingredient.
///
/// # Errors
///
/// Returns [`StoreError::Conflict`] if `alias` is already a name or alias, or a database error.
pub async fn add_alias(pool: &SqlitePool, id: i64, alias: &str) -> Result<bool, StoreError> {
    let mut tx = crate::begin_write(pool).await?;
    ensure_not_a_name(&mut tx, alias).await?;
    let result = sqlx::query(
        "INSERT INTO ingredient_aliases (alias, ingredient_id) SELECT ?, id FROM ingredients WHERE id = ?",
    )
    .bind(alias)
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(unique_or_database(alias))?;
    tx.commit().await?;
    Ok(result.rows_affected() > 0)
}

/// Removes `alias` from the ingredient `id`. Returns `false` when it was not an alias of it.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn remove_alias(pool: &SqlitePool, id: i64, alias: &str) -> Result<bool, StoreError> {
    let result =
        sqlx::query("DELETE FROM ingredient_aliases WHERE ingredient_id = ? AND alias = ?")
            .bind(id)
            .bind(alias)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}

/// Sets the store section of the ingredient `id`, or clears it with `None`. Returns `false` when
/// there is no such ingredient.
///
/// # Errors
///
/// Returns an error if the query fails, including when `section_id` does not exist.
pub async fn set_section(
    pool: &SqlitePool,
    id: i64,
    section_id: Option<i64>,
) -> Result<bool, StoreError> {
    let result = sqlx::query("UPDATE ingredients SET store_section_id = ? WHERE id = ?")
        .bind(section_id)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Approves the ingredient `id`, clearing its review flag. Returns `false` when there is no such
/// ingredient.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn approve(pool: &SqlitePool, id: i64) -> Result<bool, StoreError> {
    let result = sqlx::query("UPDATE ingredients SET pending_review = 0 WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Merges the ingredient `from` into `into`: recipe lines, shopping items and aliases move to
/// `into`, the name of `from` becomes an alias of `into`, and `from` is deleted.
///
/// Returns `false`, and changes nothing, when either ingredient is missing or both are the same.
///
/// # Errors
///
/// Returns an error if a query fails. Nothing is changed in that case.
pub async fn merge(pool: &SqlitePool, from: i64, into: i64) -> Result<bool, StoreError> {
    if from == into {
        return Ok(false);
    }
    let mut tx = crate::begin_write(pool).await?;
    let found: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ingredients WHERE id IN (?, ?)")
        .bind(from)
        .bind(into)
        .fetch_one(&mut *tx)
        .await?;
    if found != 2 {
        return Ok(false);
    }
    let from_name: String = sqlx::query_scalar("SELECT name FROM ingredients WHERE id = ?")
        .bind(from)
        .fetch_one(&mut *tx)
        .await?;
    sqlx::query("UPDATE ingredient_aliases SET ingredient_id = ? WHERE ingredient_id = ?")
        .bind(into)
        .bind(from)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO ingredient_aliases (alias, ingredient_id) VALUES (?, ?)")
        .bind(from_name)
        .bind(into)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE recipe_lines SET ingredient_id = ? WHERE ingredient_id = ?")
        .bind(into)
        .bind(from)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE shopping_items SET ingredient_id = ? WHERE ingredient_id = ?")
        .bind(into)
        .bind(from)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM ingredients WHERE id = ?")
        .bind(from)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(true)
}
