//! Changing a household's shopping list. Every query is limited to the household, so ids of other
//! households change nothing.

use crabbage_domain::quantity::{Quantity, Unit};
use crabbage_domain::scaling::{Servings, scale};
use sqlx::sqlite::SqlitePool;

use crate::StoreError;

/// Adds each planned recipe in the inclusive range once and marks the entries as listed.
///
/// # Errors
///
/// Returns an error if a query fails. The whole range is added atomically.
pub async fn add_plan(
    pool: &SqlitePool,
    household: i64,
    from: &str,
    to: &str,
) -> Result<u64, StoreError> {
    crate::planner::validate_date(from)?;
    crate::planner::validate_date(to)?;
    let mut tx = crate::begin_write(pool).await?;
    let entries: Vec<(i64, i64, i64)> = sqlx::query_as(
        "SELECT id, recipe_id, servings FROM plan_entries WHERE household_id = ? AND date >= ? AND date <= ? AND recipe_id IS NOT NULL AND on_list = 0 ORDER BY date",
    )
    .bind(household)
    .bind(from)
    .bind(to)
    .fetch_all(&mut *tx)
    .await?;
    for (entry, recipe, count) in &entries {
        let servings = valid_servings(*count).ok_or(StoreError::InvalidPlannerSettings)?;
        insert_plan_lines(&mut tx, household, *recipe, *entry, servings).await?;
        sqlx::query("UPDATE plan_entries SET on_list = 1 WHERE id = ?")
            .bind(entry)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(u64::try_from(entries.len()).unwrap_or(u64::MAX))
}

/// Adds the lines of recipe `recipe` to the list of `household`, scaled to `servings`.
///
/// Lines with an ingredient become items for it; lines without one become items for their note,
/// and are skipped when they have no note. Returns `false`, and adds nothing, when there is no
/// such recipe.
///
/// # Errors
///
/// Returns an error if a query fails. Nothing is added in that case.
pub async fn add_recipe(
    pool: &SqlitePool,
    household: i64,
    recipe: i64,
    servings: Servings,
) -> Result<bool, StoreError> {
    let mut tx = crate::begin_write(pool).await?;
    let Some(from): Option<i64> = sqlx::query_scalar("SELECT servings FROM recipes WHERE id = ?")
        .bind(recipe)
        .fetch_optional(&mut *tx)
        .await?
    else {
        return Ok(false);
    };
    let from = valid_servings(from).unwrap_or(servings);
    insert_recipe_lines(&mut tx, household, recipe, None, from, servings).await?;
    tx.commit().await?;
    Ok(true)
}

#[derive(sqlx::FromRow)]
struct LineRow {
    amount: Option<f64>,
    unit: Option<String>,
    ingredient_id: Option<i64>,
    note: Option<String>,
}

pub(crate) async fn insert_recipe_lines(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    household: i64,
    recipe: i64,
    plan_entry: Option<i64>,
    from: Servings,
    servings: Servings,
) -> Result<(), StoreError> {
    let lines: Vec<LineRow> = sqlx::query_as(
        "SELECT amount, unit, ingredient_id, note FROM recipe_lines WHERE recipe_id = ? ORDER BY position",
    )
    .bind(recipe)
    .fetch_all(&mut **tx)
    .await?;
    for line in lines {
        let (amount, unit) = scaled(line.amount, line.unit, from, servings);
        let (ingredient_id, text) = match (line.ingredient_id, line.note) {
            (Some(ingredient), _) => (Some(ingredient), None),
            (None, Some(note)) if !note.trim().is_empty() => (None, Some(note.trim().to_owned())),
            (None, _) => continue,
        };
        sqlx::query("INSERT INTO shopping_items (household_id, ingredient_id, text, amount, unit, recipe_id, plan_entry_id) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(household)
            .bind(ingredient_id)
            .bind(text)
            .bind(amount)
            .bind(unit)
            .bind(recipe)
            .bind(plan_entry)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

pub(crate) async fn insert_plan_lines(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    household: i64,
    recipe: i64,
    plan_entry: i64,
    servings: Servings,
) -> Result<(), StoreError> {
    let recipe_servings: Option<i64> =
        sqlx::query_scalar("SELECT servings FROM recipes WHERE id = ?")
            .bind(recipe)
            .fetch_optional(&mut **tx)
            .await?;
    let from = recipe_servings.and_then(valid_servings).unwrap_or(servings);
    insert_recipe_lines(tx, household, recipe, Some(plan_entry), from, servings).await
}

/// Returns valid servings when the stored value is positive and fits in `u32`.
pub(crate) fn valid_servings(value: i64) -> Option<Servings> {
    u32::try_from(value).ok().and_then(Servings::new)
}

/// The amount and unit symbol of a recipe line scaled from `from` to `to` servings. A unitless
/// amount scales as pieces and keeps no unit. An amount whose unit is unknown is kept as it is.
fn scaled(
    amount: Option<f64>,
    unit: Option<String>,
    from: Servings,
    to: Servings,
) -> (Option<f64>, Option<String>) {
    let Some(amount) = amount else {
        return (None, unit);
    };
    let measure = unit.as_deref().map_or(Some(Unit::Piece), Unit::from_symbol);
    let scaled = measure
        .and_then(|measure| Quantity::new(amount, measure))
        .map(|quantity| scale(quantity, from, to));
    match scaled {
        Some(quantity) if unit.is_some() => (
            Some(quantity.amount()),
            Some(quantity.unit().symbol().to_owned()),
        ),
        Some(quantity) => (Some(quantity.amount()), None),
        None => (Some(amount), unit),
    }
}

/// Adds a manual item with `text` to the list of `household` and returns its id. The text is
/// trimmed.
///
/// # Errors
///
/// Returns [`StoreError::Empty`] if the text is blank, or a database error.
pub async fn add_text(pool: &SqlitePool, household: i64, text: &str) -> Result<i64, StoreError> {
    let text = non_blank(text)?;
    let id = sqlx::query_scalar(
        "INSERT INTO shopping_items (household_id, text) VALUES (?, ?) RETURNING id",
    )
    .bind(household)
    .bind(text)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

/// Replaces the text of the manual item `id` of `household` with `text`, trimmed. Returns `false`
/// when there is no such manual item: items for an ingredient keep their ingredient.
///
/// # Errors
///
/// Returns [`StoreError::Empty`] if the text is blank, or a database error.
pub async fn edit_text(
    pool: &SqlitePool,
    household: i64,
    id: i64,
    text: &str,
) -> Result<bool, StoreError> {
    let text = non_blank(text)?;
    let result = sqlx::query(
        "UPDATE shopping_items SET text = ? \
         WHERE id = ? AND household_id = ? AND text IS NOT NULL",
    )
    .bind(text)
    .bind(id)
    .bind(household)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Removes the item `id` from the list of `household`. Returns `false` when there is no such item.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn remove(pool: &SqlitePool, household: i64, id: i64) -> Result<bool, StoreError> {
    let result = sqlx::query("DELETE FROM shopping_items WHERE id = ? AND household_id = ?")
        .bind(id)
        .bind(household)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Sets the checked state of the items `ids` of `household`. Returns the number of items
/// changed; ids of other households are ignored.
///
/// # Errors
///
/// Returns an error if a query fails. Nothing is changed in that case.
pub async fn set_checked(
    pool: &SqlitePool,
    household: i64,
    ids: &[i64],
    checked: bool,
) -> Result<u64, StoreError> {
    let mut tx = crate::begin_write(pool).await?;
    let mut changed = 0;
    for id in ids {
        let result =
            sqlx::query("UPDATE shopping_items SET checked = ? WHERE id = ? AND household_id = ?")
                .bind(checked)
                .bind(id)
                .bind(household)
                .execute(&mut *tx)
                .await?;
        changed += result.rows_affected();
    }
    tx.commit().await?;
    Ok(changed)
}

/// Removes the checked items of `household` and returns how many were removed.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn clear_checked(pool: &SqlitePool, household: i64) -> Result<u64, StoreError> {
    let result = sqlx::query("DELETE FROM shopping_items WHERE household_id = ? AND checked = 1")
        .bind(household)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

fn non_blank(text: &str) -> Result<&str, StoreError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        Err(StoreError::Empty)
    } else {
        Ok(trimmed)
    }
}
