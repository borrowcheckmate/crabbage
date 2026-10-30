//! Reading recipes: one with its lines, steps, owner and editors, and the list.

use sqlx::FromRow;
use sqlx::sqlite::SqlitePool;

use super::model::{Person, Recipe, RecipeStep, RecipeSummary};
use crate::StoreError;

#[derive(FromRow)]
struct Header {
    id: i64,
    title: String,
    servings: i64,
    owner_id: i64,
    owner_name: String,
}

#[derive(FromRow)]
struct StepRow {
    id: i64,
    text: String,
    duration_seconds: Option<i64>,
}

/// The recipe `id`, or `None` when there is no such recipe.
///
/// # Errors
///
/// Returns an error if a query fails.
pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<Recipe>, StoreError> {
    let header: Option<Header> = sqlx::query_as(
        "SELECT r.id, r.title, r.servings, r.owner_id, u.name AS owner_name \
         FROM recipes r JOIN users u ON u.id = r.owner_id WHERE r.id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    let Some(header) = header else {
        return Ok(None);
    };
    let editors = sqlx::query_as(
        "SELECT u.id, u.name FROM recipe_editors e JOIN users u ON u.id = e.user_id \
         WHERE e.recipe_id = ? ORDER BY u.name COLLATE NOCASE, u.id",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;
    let lines = sqlx::query_as(
        "SELECT l.id, l.amount, l.unit, i.name AS ingredient, l.note FROM recipe_lines l \
         LEFT JOIN ingredients i ON i.id = l.ingredient_id \
         WHERE l.recipe_id = ? ORDER BY l.position",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;
    let steps = steps_of(pool, id).await?;
    Ok(Some(Recipe {
        id: header.id,
        title: header.title,
        servings: header.servings,
        owner_id: header.owner_id,
        owner_name: header.owner_name,
        editors,
        lines,
        steps,
    }))
}

async fn steps_of(pool: &SqlitePool, recipe_id: i64) -> Result<Vec<RecipeStep>, StoreError> {
    let rows: Vec<StepRow> = sqlx::query_as(
        "SELECT id, text, duration_seconds FROM steps WHERE recipe_id = ? ORDER BY position",
    )
    .bind(recipe_id)
    .fetch_all(pool)
    .await?;
    let links: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT sl.step_id, sl.line_id FROM step_lines sl \
         JOIN recipe_lines rl ON rl.id = sl.line_id \
         WHERE rl.recipe_id = ? ORDER BY rl.position",
    )
    .bind(recipe_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| RecipeStep {
            line_ids: links
                .iter()
                .filter(|(step, _)| *step == row.id)
                .map(|(_, line)| *line)
                .collect(),
            text: row.text,
            duration_seconds: row.duration_seconds,
        })
        .collect())
}

/// Every recipe, ordered by title.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn list(pool: &SqlitePool) -> Result<Vec<RecipeSummary>, StoreError> {
    let recipes = sqlx::query_as(
        "SELECT r.id, r.title, r.servings, u.name AS owner_name FROM recipes r \
         JOIN users u ON u.id = r.owner_id ORDER BY r.title COLLATE NOCASE, r.id",
    )
    .fetch_all(pool)
    .await?;
    Ok(recipes)
}

/// Every user, as a person to pick an editor from, ordered by name.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn list_users(pool: &SqlitePool) -> Result<Vec<Person>, StoreError> {
    let people = sqlx::query_as("SELECT id, name FROM users ORDER BY name COLLATE NOCASE, id")
        .fetch_all(pool)
        .await?;
    Ok(people)
}
