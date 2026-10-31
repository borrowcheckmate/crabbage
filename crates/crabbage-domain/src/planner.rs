use fastrand::Rng;

/// A meal category for a day, or a day with no planned meal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DayPlan {
    /// The day has no meal.
    Rest,
    /// The day takes a recipe from this category.
    Category(i64),
}

/// A day number and its requested meal plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanningDay {
    /// The day number, ordered chronologically with other input days.
    pub day: i64,
    /// The category or rest-day selection.
    pub plan: DayPlan,
}

/// A recipe and the category it belongs to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Candidate {
    /// The recipe identifier.
    pub recipe_id: i64,
    /// The category identifier.
    pub category_id: i64,
}

/// A record of a recipe planned on a day.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HistoryEntry {
    /// The recipe identifier.
    pub recipe_id: i64,
    /// The day number.
    pub day: i64,
}

/// Selects one recipe for each category day, preserving rest days as `None`.
///
/// A recipe planned within the lookback window is excluded. If all recipes in
/// the category are excluded, the least recently planned recipe is selected.
pub fn plan(
    days: &[PlanningDay],
    candidates: &[Candidate],
    history: &[HistoryEntry],
    lookback_weeks: u32,
    rng: &mut Rng,
) -> Vec<Option<i64>> {
    let lookback_days = i64::from(lookback_weeks) * 7;
    let mut recent = history.to_vec();
    let mut picked_recipes = std::collections::BTreeSet::new();
    let mut result = Vec::with_capacity(days.len());

    for planning_day in days {
        let DayPlan::Category(category_id) = planning_day.plan else {
            result.push(None);
            continue;
        };

        let mut category: Vec<Candidate> = candidates
            .iter()
            .copied()
            .filter(|candidate| candidate.category_id == category_id)
            .collect();
        category.sort_by_key(|candidate| candidate.recipe_id);
        category.dedup_by_key(|candidate| candidate.recipe_id);

        if category.is_empty() {
            result.push(None);
            continue;
        }

        let eligible: Vec<Candidate> = category
            .iter()
            .copied()
            .filter(|candidate| {
                !picked_recipes.contains(&candidate.recipe_id)
                    && !recent.iter().any(|entry| {
                        entry.recipe_id == candidate.recipe_id
                            && planning_day.day.saturating_sub(entry.day) >= 0
                            && planning_day.day.saturating_sub(entry.day) <= lookback_days
                    })
            })
            .collect();

        let picked = if eligible.is_empty() {
            category
                .iter()
                .min_by_key(|candidate| {
                    (
                        recent
                            .iter()
                            .filter(|entry| entry.recipe_id == candidate.recipe_id)
                            .map(|entry| entry.day)
                            .max(),
                        candidate.recipe_id,
                    )
                })
                .map(|candidate| candidate.recipe_id)
        } else {
            let index = rng.usize(0..eligible.len());
            eligible.get(index).map(|candidate| candidate.recipe_id)
        };

        if let Some(recipe_id) = picked {
            picked_recipes.insert(recipe_id);
            recent.push(HistoryEntry {
                recipe_id,
                day: planning_day.day,
            });
        }
        result.push(picked);
    }

    result
}
