//! Creating, replacing and deleting recipes. Creating and replacing run in one transaction.

use sqlx::{SqliteConnection, sqlite::SqlitePool};

use super::model::{LineInput, RecipeInput};
use crate::StoreError;

/// Creates a recipe owned by `owner_id`, created at `now` (unix seconds), and returns its id.
///
/// # Errors
///
/// Returns [`StoreError::UnknownLine`] when a step links to a line that is not in the recipe, or a
/// database error.
pub async fn create(
    pool: &SqlitePool,
    owner_id: i64,
    input: &RecipeInput,
    now: i64,
) -> Result<i64, StoreError> {
    let mut tx = pool.begin().await?;
    let id = sqlx::query_scalar(
        "INSERT INTO recipes (owner_id, title, servings, created_at) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(owner_id)
    .bind(&input.title)
    .bind(input.servings)
    .bind(now)
    .fetch_one(&mut *tx)
    .await?;
    write_contents(&mut tx, id, input).await?;
    tx.commit().await?;
    Ok(id)
}

/// Replaces the title, servings, categories, lines and steps of recipe `id`. Returns `false`, and changes
/// nothing, when there is no such recipe.
///
/// # Errors
///
/// Returns [`StoreError::UnknownLine`] when a step links to a line that is not in the recipe, or a
/// database error. Nothing is changed in that case.
pub async fn update(pool: &SqlitePool, id: i64, input: &RecipeInput) -> Result<bool, StoreError> {
    let mut tx = pool.begin().await?;
    let found = sqlx::query("UPDATE recipes SET title = ?, servings = ? WHERE id = ?")
        .bind(&input.title)
        .bind(input.servings)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    if found.rows_affected() == 0 {
        return Ok(false);
    }
    sqlx::query("DELETE FROM recipe_categories WHERE recipe_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM steps WHERE recipe_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM recipe_lines WHERE recipe_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    write_contents(&mut tx, id, input).await?;
    tx.commit().await?;
    Ok(true)
}

/// Deletes recipe `id` with its lines, steps, editors, ratings and favourites. Returns `false` when
/// there is no such recipe.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, StoreError> {
    let result = sqlx::query("DELETE FROM recipes WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Files recipe `recipe_id` under its categories, then writes the lines and the steps. The
/// categories and the lines must not exist yet.
async fn write_contents(
    conn: &mut SqliteConnection,
    recipe_id: i64,
    input: &RecipeInput,
) -> Result<(), StoreError> {
    for &category_id in &input.category_ids {
        sqlx::query(
            "INSERT OR IGNORE INTO recipe_categories (recipe_id, category_id) \
             SELECT ?, id FROM categories WHERE id = ?",
        )
        .bind(recipe_id)
        .bind(category_id)
        .execute(&mut *conn)
        .await?;
    }
    let mut line_ids = Vec::with_capacity(input.lines.len());
    for (position, line) in (0_i64..).zip(&input.lines) {
        let line_id = write_line(conn, recipe_id, position, line).await?;
        line_ids.push(line_id);
    }
    for (position, step) in (0_i64..).zip(&input.steps) {
        let step_id: i64 = sqlx::query_scalar(
            "INSERT INTO steps (recipe_id, position, text, duration_seconds) VALUES (?, ?, ?, ?) RETURNING id",
        )
        .bind(recipe_id)
        .bind(position)
        .bind(&step.text)
        .bind(step.duration_seconds)
        .fetch_one(&mut *conn)
        .await?;
        for &line_position in &step.line_positions {
            let line_id = line_ids
                .get(line_position)
                .copied()
                .ok_or(StoreError::UnknownLine)?;
            sqlx::query("INSERT OR IGNORE INTO step_lines (step_id, line_id) VALUES (?, ?)")
                .bind(step_id)
                .bind(line_id)
                .execute(&mut *conn)
                .await?;
        }
    }
    Ok(())
}

async fn write_line(
    conn: &mut SqliteConnection,
    recipe_id: i64,
    position: i64,
    line: &LineInput,
) -> Result<i64, StoreError> {
    let ingredient_id = match line.ingredient.as_deref().map(str::trim) {
        Some(name) if !name.is_empty() => Some(ingredient_id(conn, name).await?),
        Some(_) | None => None,
    };
    let id = sqlx::query_scalar(
        "INSERT INTO recipe_lines (recipe_id, position, amount, unit, ingredient_id, note) \
         VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(recipe_id)
    .bind(position)
    .bind(line.amount)
    .bind(line.unit.as_deref())
    .bind(ingredient_id)
    .bind(line.note.as_deref())
    .fetch_one(conn)
    .await?;
    Ok(id)
}

/// The id of the ingredient whose name or alias is `name`, ignoring case. An unknown name creates
/// an ingredient waiting for review.
async fn ingredient_id(conn: &mut SqliteConnection, name: &str) -> Result<i64, StoreError> {
    let known: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM ingredients WHERE name = ?1 \
         UNION ALL SELECT ingredient_id FROM ingredient_aliases WHERE alias = ?1 LIMIT 1",
    )
    .bind(name)
    .fetch_optional(&mut *conn)
    .await?;
    if let Some(id) = known {
        return Ok(id);
    }
    let id = sqlx::query_scalar(
        "INSERT INTO ingredients (name, pending_review) VALUES (?, 1) RETURNING id",
    )
    .bind(name)
    .fetch_one(conn)
    .await?;
    Ok(id)
}
