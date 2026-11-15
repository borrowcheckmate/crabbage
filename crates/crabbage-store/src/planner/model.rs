//! Planner records returned by the store and accepted for writes.

use crabbage_domain::allergens::Allergen;
use crabbage_domain::meal_types::MealType;
use crabbage_domain::warnings::Warning;
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
    /// Meal types the planner picks from, where none means any. Untyped recipes always qualify.
    pub meal_types: Vec<MealType>,
}

/// What a household avoids. A recipe conflicts when an ingredient has one of the allergens or the
/// recipe has one of the warnings.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Avoid {
    /// The avoided allergens, in [`Allergen::ALL`] order.
    pub allergens: Vec<Allergen>,
    /// The avoided dietary warnings, in [`Warning::ALL`] order.
    pub warnings: Vec<Warning>,
}

impl Avoid {
    /// The codes of the avoided allergens, then of the avoided warnings.
    pub fn codes(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.allergens
            .iter()
            .map(|allergen| allergen.code())
            .chain(self.warnings.iter().map(|warning| warning.code()))
    }
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
