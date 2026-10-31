//! Reading household planner settings, entries and recipe history.

use sqlx::sqlite::SqlitePool;

use super::model::{PlanEntry, PlannerSettings};
use crate::StoreError;

/// Returns saved settings, or the planner defaults when none have been saved.
///
/// # Errors
///
/// Returns an error if a query fails.
pub async fn settings(pool: &SqlitePool, household: i64) -> Result<PlannerSettings, StoreError> {
    let saved: Option<(i64, i64)> = sqlx::query_as(
        "SELECT lookback_weeks, servings FROM planner_settings WHERE household_id = ?",
    )
    .bind(household)
    .fetch_optional(pool)
    .await?;
    let weekdays: Vec<(i64, Option<i64>)> = sqlx::query_as(
        "SELECT weekday, category_id FROM planner_weekdays WHERE household_id = ? ORDER BY weekday",
    )
    .bind(household)
    .fetch_all(pool)
    .await?;
    let mut result = PlannerSettings {
        lookback_weeks: saved.map_or(1, |setting| setting.0),
        servings: saved.map_or(4, |setting| setting.1),
        weekdays: [None; 7],
    };
    for (weekday, category) in weekdays {
        if let Ok(index) = usize::try_from(weekday)
            && let Some(slot) = result.weekdays.get_mut(index)
        {
            *slot = category;
        }
    }
    Ok(result)
}

/// Returns entries for the inclusive ISO date range, ordered by date.
///
/// # Errors
///
/// Returns [`StoreError::InvalidDate`] for an invalid boundary, or a database error.
pub async fn entries(
    pool: &SqlitePool,
    household: i64,
    from: &str,
    to: &str,
) -> Result<Vec<PlanEntry>, StoreError> {
    crate::planner::write::validate_date(from)?;
    crate::planner::write::validate_date(to)?;
    let result = sqlx::query_as(
        "SELECT date, recipe_id, servings FROM plan_entries \
         WHERE household_id = ? AND date >= ? AND date <= ? ORDER BY date",
    )
    .bind(household)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;
    Ok(result)
}

/// Returns non-rest recipes planned in the inclusive range from `since_date` to `before_date`.
///
/// # Errors
///
/// Returns [`StoreError::InvalidDate`] for an invalid boundary, or a database error.
pub async fn history(
    pool: &SqlitePool,
    household: i64,
    before_date: &str,
    since_date: &str,
) -> Result<Vec<(i64, String)>, StoreError> {
    crate::planner::write::validate_date(before_date)?;
    crate::planner::write::validate_date(since_date)?;
    let result = sqlx::query_as(
        "SELECT recipe_id, date FROM plan_entries \
         WHERE household_id = ? AND recipe_id IS NOT NULL AND date >= ? AND date <= ? ORDER BY date",
    )
    .bind(household)
    .bind(since_date)
    .bind(before_date)
    .fetch_all(pool)
    .await?;
    Ok(result)
}
