//! Saving household planner settings and dated entries.

use sqlx::sqlite::SqlitePool;

use super::model::{PlanEntry, PlannerSettings};
use crate::{StoreError, begin_write};

/// Saves planner settings and the category or rest-day choice for every weekday.
///
/// # Errors
///
/// Returns [`StoreError::InvalidPlannerSettings`] when lookback is negative or servings are not
/// positive, or a database error. The settings are changed atomically.
pub async fn save_settings(
    pool: &SqlitePool,
    household: i64,
    settings: &PlannerSettings,
) -> Result<(), StoreError> {
    if settings.lookback_weeks < 0 || settings.servings <= 0 {
        return Err(StoreError::InvalidPlannerSettings);
    }
    let mut tx = begin_write(pool).await?;
    sqlx::query(
        "INSERT INTO planner_settings (household_id, lookback_weeks, servings) VALUES (?, ?, ?) \
         ON CONFLICT (household_id) DO UPDATE SET lookback_weeks = excluded.lookback_weeks, servings = excluded.servings",
    )
    .bind(household)
    .bind(settings.lookback_weeks)
    .bind(settings.servings)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM planner_weekdays WHERE household_id = ?")
        .bind(household)
        .execute(&mut *tx)
        .await?;
    for (weekday, category) in settings.weekdays.iter().enumerate() {
        sqlx::query(
            "INSERT INTO planner_weekdays (household_id, weekday, category_id) VALUES (?, ?, ?)",
        )
        .bind(household)
        .bind(i64::try_from(weekday).map_err(|_error| StoreError::InvalidPlannerSettings)?)
        .bind(category)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Inserts or replaces a recipe or rest entry for a household date.
///
/// # Errors
///
/// Returns [`StoreError::InvalidDate`] for an invalid ISO date, or a database error.
pub async fn set_entry(
    pool: &SqlitePool,
    household: i64,
    entry: &PlanEntry,
) -> Result<(), StoreError> {
    validate_date(&entry.date)?;
    if entry.servings <= 0 {
        return Err(StoreError::InvalidPlannerSettings);
    }
    let mut tx = begin_write(pool).await?;
    let previous: Option<(i64, Option<i64>, i64)> = sqlx::query_as(
        "SELECT id, recipe_id, servings FROM plan_entries WHERE household_id = ? AND date = ? AND on_list = 1",
    )
    .bind(household)
    .bind(&entry.date)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some((id, _, _)) = previous {
        sqlx::query("DELETE FROM shopping_items WHERE plan_entry_id = ? AND checked = 0")
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query(
        "INSERT INTO plan_entries (household_id, date, recipe_id, servings) VALUES (?, ?, ?, ?) \
         ON CONFLICT (household_id, date) DO UPDATE SET recipe_id = excluded.recipe_id, servings = excluded.servings",
    )
    .bind(household)
    .bind(&entry.date)
    .bind(entry.recipe_id)
    .bind(entry.servings)
    .execute(&mut *tx)
    .await?;
    if let Some((id, _, _)) = previous
        && let Some(recipe) = entry.recipe_id
    {
        let to = crate::shopping::valid_servings(entry.servings)
            .ok_or(StoreError::InvalidPlannerSettings)?;
        crate::shopping::insert_plan_lines(&mut tx, household, recipe, id, to).await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Removes the entry for `date` from `household` and reports whether one existed.
///
/// # Errors
///
/// Returns [`StoreError::InvalidDate`] for an invalid date, or a database error.
pub async fn remove_entry(
    pool: &SqlitePool,
    household: i64,
    date: &str,
) -> Result<bool, StoreError> {
    validate_date(date)?;
    let mut tx = begin_write(pool).await?;
    let id: Option<i64> =
        sqlx::query_scalar("SELECT id FROM plan_entries WHERE household_id = ? AND date = ?")
            .bind(household)
            .bind(date)
            .fetch_optional(&mut *tx)
            .await?;
    if let Some(id) = id {
        sqlx::query("UPDATE shopping_items SET plan_entry_id = NULL WHERE plan_entry_id = ? AND checked = 1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    let result = sqlx::query("DELETE FROM plan_entries WHERE household_id = ? AND date = ?")
        .bind(household)
        .bind(date)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(result.rows_affected() > 0)
}

pub(crate) fn validate_date(value: &str) -> Result<(), StoreError> {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || bytes
            .iter()
            .enumerate()
            .any(|(index, byte)| index != 4 && index != 7 && !byte.is_ascii_digit())
    {
        return Err(StoreError::InvalidDate);
    }
    let year = value
        .get(0..4)
        .ok_or(StoreError::InvalidDate)?
        .parse::<i32>()
        .map_err(|_error| StoreError::InvalidDate)?;
    let month = value
        .get(5..7)
        .ok_or(StoreError::InvalidDate)?
        .parse::<u32>()
        .map_err(|_error| StoreError::InvalidDate)?;
    let day = value
        .get(8..10)
        .ok_or(StoreError::InvalidDate)?
        .parse::<u32>()
        .map_err(|_error| StoreError::InvalidDate)?;
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    if year == 0 || day == 0 || day > days {
        return Err(StoreError::InvalidDate);
    }
    Ok(())
}
