//! Recipe and category lookups used by the planner.

use crabbage_domain::meal_types::MealType;
use sqlx::QueryBuilder;
use sqlx::sqlite::{Sqlite, SqlitePool};

use crate::StoreError;

fn push_meal_filter(query: &mut QueryBuilder<Sqlite>, recipe: &str, meal_types: &[MealType]) {
    if meal_types.is_empty() {
        return;
    }
    query.push(format!(
        " AND ({recipe} NOT IN (SELECT recipe_id FROM recipe_meal_types) \
         OR {recipe} IN (SELECT recipe_id FROM recipe_meal_types WHERE meal_type IN ("
    ));
    let mut separated = query.separated(", ");
    for meal_type in meal_types {
        separated.push_bind(meal_type.code());
    }
    separated.push_unseparated(")))");
}

/// Returns the `(category, recipe)` pairs of the recipes filed under `categories`, ordered by
/// category and recipe id. With `meal_types`, only recipes of one of them or of none qualify.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn recipes_in_categories(
    pool: &SqlitePool,
    categories: &[i64],
    meal_types: &[MealType],
) -> Result<Vec<(i64, i64)>, StoreError> {
    if categories.is_empty() {
        return Ok(Vec::new());
    }
    let mut query = QueryBuilder::new(
        "SELECT category_id, recipe_id FROM recipe_categories WHERE category_id IN (",
    );
    let mut separated = query.separated(", ");
    for category in categories {
        separated.push_bind(category);
    }
    separated.push_unseparated(")");
    push_meal_filter(&mut query, "recipe_id", meal_types);
    query.push(" ORDER BY category_id, recipe_id");
    Ok(query.build_query_as().fetch_all(pool).await?)
}

/// Returns every recipe id, ordered by id. With `meal_types`, only recipes of one of them or of
/// none qualify.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn recipe_ids(
    pool: &SqlitePool,
    meal_types: &[MealType],
) -> Result<Vec<i64>, StoreError> {
    let mut query = QueryBuilder::new("SELECT id FROM recipes WHERE 1");
    push_meal_filter(&mut query, "id", meal_types);
    query.push(" ORDER BY id");
    Ok(query.build_query_scalar().fetch_all(pool).await?)
}

/// Returns every recipe id and title, ordered by title ignoring case, then id. With `meal_types`,
/// only recipes of one of them or of none qualify.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn recipe_titles(
    pool: &SqlitePool,
    meal_types: &[MealType],
) -> Result<Vec<(i64, String)>, StoreError> {
    let mut query = QueryBuilder::new("SELECT id, title FROM recipes WHERE 1");
    push_meal_filter(&mut query, "id", meal_types);
    query.push(" ORDER BY title COLLATE NOCASE, id");
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
    let mut query = QueryBuilder::new("SELECT COUNT(DISTINCT id) FROM categories WHERE id IN (");
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
