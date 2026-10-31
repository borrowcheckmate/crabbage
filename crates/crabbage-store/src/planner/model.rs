//! Planner records returned by the store and accepted for writes.

use sqlx::FromRow;

/// Household planner preferences. Weekdays use Monday as index zero; `None` means rest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannerSettings {
    /// Number of weeks to avoid repeating recipes, where zero disables lookback.
    pub lookback_weeks: i64,
    /// Default servings for newly planned entries.
    pub servings: i64,
    /// Category for each weekday, with Monday at index zero and `None` for a rest day.
    pub weekdays: [Option<i64>; 7],
}

/// A household's recipe or rest entry for one ISO calendar date.
#[derive(Clone, Debug, Eq, PartialEq, FromRow)]
pub struct PlanEntry {
    /// Date in `YYYY-MM-DD` form.
    pub date: String,
    /// Planned recipe, or `None` for a rest day.
    pub recipe_id: Option<i64>,
    /// Servings planned for the date.
    pub servings: i64,
}
