//! Meal planner behaviour.

use crabbage_domain::planner::{Candidate, DayPlan, HistoryEntry, PlanningDay, plan};
use fastrand::Rng;

const fn day(day: i64, plan: DayPlan) -> PlanningDay {
    PlanningDay { day, plan }
}

const fn candidate(recipe_id: i64, category_id: i64) -> Candidate {
    Candidate {
        recipe_id,
        category_id,
    }
}

const fn history(recipe_id: i64, day: i64) -> HistoryEntry {
    HistoryEntry { recipe_id, day }
}

#[test]
fn picks_only_from_the_days_category() {
    let mut rng = Rng::with_seed(1);
    let selected = plan(
        &[day(100, DayPlan::Category(4))],
        &[candidate(1, 3), candidate(2, 4)],
        &[],
        1,
        &mut rng,
    );

    assert_eq!(selected, vec![Some(2)]);
}

#[test]
fn excludes_recipes_planned_within_the_lookback() {
    let mut rng = Rng::with_seed(2);
    let selected = plan(
        &[day(100, DayPlan::Category(4))],
        &[candidate(1, 4), candidate(2, 4)],
        &[history(1, 94)],
        1,
        &mut rng,
    );

    assert_eq!(selected, vec![Some(2)]);
}

#[test]
fn falls_back_to_the_least_recently_planned_recipe() {
    let mut rng = Rng::with_seed(3);
    let selected = plan(
        &[day(100, DayPlan::Category(4))],
        &[candidate(1, 4), candidate(2, 4), candidate(3, 4)],
        &[history(1, 99), history(2, 95)],
        1,
        &mut rng,
    );

    assert_eq!(selected, vec![Some(3)]);
}

#[test]
fn rest_days_get_none() {
    let mut rng = Rng::with_seed(4);
    let selected = plan(
        &[day(100, DayPlan::Rest)],
        &[candidate(1, 4)],
        &[],
        1,
        &mut rng,
    );

    assert_eq!(selected, vec![None]);
}

#[test]
fn empty_categories_get_none() {
    let mut rng = Rng::with_seed(5);
    let selected = plan(&[day(100, DayPlan::Category(4))], &[], &[], 1, &mut rng);

    assert_eq!(selected, vec![None]);
}

#[test]
fn avoids_repeats_within_a_plan_when_enough_candidates_exist() {
    let mut rng = Rng::with_seed(6);
    let selected = plan(
        &[
            day(100, DayPlan::Category(4)),
            day(101, DayPlan::Category(4)),
            day(102, DayPlan::Category(4)),
        ],
        &[candidate(1, 4), candidate(2, 4), candidate(3, 4)],
        &[],
        1,
        &mut rng,
    );

    let recipes: Vec<i64> = selected.into_iter().flatten().collect();
    assert_eq!(recipes.len(), 3);
    assert_ne!(recipes[0], recipes[1]);
    assert_ne!(recipes[0], recipes[2]);
    assert_ne!(recipes[1], recipes[2]);
}
