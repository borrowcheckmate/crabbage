//! Recipe and category lookups used by the planner.

use sqlx::sqlite::SqlitePool;

use crate::StoreError;

/// Returns the `(category, recipe)` pairs of the recipes filed under `categories`, ordered by
/// category and recipe id.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn recipes_in_categories(
    pool: &SqlitePool,
    categories: &[i64],
) -> Result<Vec<(i64, i64)>, StoreError> {
    if categories.is_empty() {
        return Ok(Vec::new());
    }
    let mut query = sqlx::QueryBuilder::new(
        "SELECT category_id, recipe_id FROM recipe_categories WHERE category_id IN (",
    );
    let mut separated = query.separated(", ");
    for category in categories {
        separated.push_bind(category);
    }
    separated.push_unseparated(") ORDER BY category_id, recipe_id");
    Ok(query.build_query_as().fetch_all(pool).await?)
}

/// Returns whether a recipe exists.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn recipe_exists(pool: &SqlitePool, recipe: i64) -> Result<bool, StoreError> {
    let result: Option<i64> = sqlx::query_scalar("SELECT id FROM recipes WHERE id = ?")
        .bind(recipe)
        .fetch_optional(pool)
        .await?;
    Ok(result.is_some())
}

/// Returns whether every selected category exists, checking all weekdays in one query.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn categories_exist(
    pool: &SqlitePool,
    categories: &[Option<i64>; 7],
) -> Result<bool, StoreError> {
    let selected: Vec<i64> = categories.iter().flatten().copied().collect();
    if selected.is_empty() {
        return Ok(true);
    }
    let mut query =
        sqlx::QueryBuilder::new("SELECT COUNT(DISTINCT id) FROM categories WHERE id IN (");
    let mut separated = query.separated(", ");
    for category in &selected {
        separated.push_bind(category);
    }
    separated.push_unseparated(")");
    let count = query.build_query_scalar::<i64>().fetch_one(pool).await?;
    let unique = selected
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    Ok(usize::try_from(count).ok() == Some(unique.len()))
}
