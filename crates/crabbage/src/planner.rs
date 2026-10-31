use axum::Router;
use axum::extract::{Extension, Form, Path, State};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::post;
use crabbage_domain::dates::{format_iso_date, parse_iso_date};
use crabbage_domain::planner::{Candidate, DayPlan, HistoryEntry, PlanningDay};
use crabbage_store::planner::{self, PlanEntry, PlannerSettings};
use serde::Deserialize;

use crate::SessionUser;
use crate::access::internal_error;
use crate::state::AppState;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/plan/generate", post(generate))
        .route("/plan/{date}/reroll", post(reroll))
        .route("/plan/{date}/recipe", post(recipe))
        .route("/plan/{date}/servings", post(servings))
        .route("/plan/settings", post(settings))
}

#[derive(Deserialize)]
struct WeekForm {
    week: String,
}

#[derive(Deserialize)]
struct RecipeForm {
    recipe: String,
}

#[derive(Deserialize)]
struct ServingsForm {
    servings: String,
}

#[derive(Deserialize)]
struct SettingsForm {
    lookback_weeks: String,
    servings: String,
    monday: String,
    tuesday: String,
    wednesday: String,
    thursday: String,
    friday: String,
    saturday: String,
    sunday: String,
}

async fn household(state: &AppState, user: &SessionUser) -> Result<i64, StatusCode> {
    crabbage_store::households::active_household(&state.pool, user.0)
        .await
        .map_err(internal_error)?
        .ok_or(StatusCode::NOT_FOUND)
}

fn week_start(value: &str) -> Result<i64, StatusCode> {
    let day = parse_iso_date(value).ok_or(StatusCode::BAD_REQUEST)?;
    Ok(day - (day + 3).rem_euclid(7))
}

fn redirect_week(value: &str) -> Result<Redirect, StatusCode> {
    let start = week_start(value)?;
    let date = format_iso_date(start).ok_or(StatusCode::BAD_REQUEST)?;
    Ok(Redirect::to(&format!("/plan?week={date}")))
}

async fn generate(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
    Form(form): Form<WeekForm>,
) -> Result<Redirect, StatusCode> {
    fill_week(&state, &user, &form.week).await?;
    redirect_week(&form.week)
}

async fn fill_week(state: &AppState, user: &SessionUser, week: &str) -> Result<(), StatusCode> {
    let start = week_start(week)?;
    let household = household(state, user).await?;
    let settings = planner::settings(&state.pool, household)
        .await
        .map_err(internal_error)?;
    let existing = planner::entries(&state.pool, household, &date(start)?, &date(start + 6)?)
        .await
        .map_err(internal_error)?;
    let days = planned_days(start, &existing, &settings);
    let history_days = settings.lookback_weeks.saturating_mul(7);
    let history_start = start.saturating_sub(history_days);
    let history = planner::history(
        &state.pool,
        household,
        &date(start - 1)?,
        &date(history_start)?,
    )
    .await
    .map_err(internal_error)?;
    let history = history
        .into_iter()
        .filter_map(|(recipe_id, day)| {
            parse_iso_date(&day).map(|day| HistoryEntry { recipe_id, day })
        })
        .collect::<Vec<_>>();
    let candidates = candidates(&state.pool, &settings).await?;
    let mut rng = fastrand::Rng::with_seed(random_seed());
    let picks = crabbage_domain::planner::plan(
        &days,
        &candidates,
        &history,
        u32::try_from(settings.lookback_weeks).unwrap_or(u32::MAX),
        &mut rng,
    );
    for (day, recipe_id) in days.iter().zip(picks) {
        if recipe_id.is_none() && matches!(day.plan, DayPlan::Category(_)) {
            continue;
        }
        planner::set_entry(
            &state.pool,
            household,
            &PlanEntry {
                date: date(day.day)?,
                recipe_id,
                servings: settings.servings,
            },
        )
        .await
        .map_err(internal_error)?;
    }
    Ok(())
}

fn planned_days(
    start: i64,
    existing: &[PlanEntry],
    settings: &PlannerSettings,
) -> Vec<PlanningDay> {
    (0_i64..7)
        .filter(|offset| {
            !existing
                .iter()
                .any(|entry| entry.date == date(start + offset).unwrap_or_default())
        })
        .filter_map(|offset| {
            settings
                .weekdays
                .get(weekday(start + offset))
                .map(|category| (offset, category))
        })
        .map(|(offset, category)| PlanningDay {
            day: start + offset,
            plan: category.map_or(DayPlan::Rest, DayPlan::Category),
        })
        .collect()
}

async fn candidates(
    pool: &sqlx::SqlitePool,
    settings: &PlannerSettings,
) -> Result<Vec<Candidate>, StatusCode> {
    let mut candidates = Vec::new();
    for category in settings.weekdays.iter().flatten() {
        let ids: Vec<i64> = sqlx::query_scalar(
            "SELECT recipe_id FROM recipe_categories WHERE category_id = ? ORDER BY recipe_id",
        )
        .bind(category)
        .fetch_all(pool)
        .await
        .map_err(internal_error)?;
        candidates.extend(ids.into_iter().map(|recipe_id| Candidate {
            recipe_id,
            category_id: *category,
        }));
    }
    Ok(candidates)
}

async fn reroll(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
    Path(day): Path<String>,
) -> Result<Redirect, StatusCode> {
    let day_number = parse_iso_date(&day).ok_or(StatusCode::BAD_REQUEST)?;
    let household = household(&state, &user).await?;
    let settings = planner::settings(&state.pool, household)
        .await
        .map_err(internal_error)?;
    let category = settings
        .weekdays
        .get(weekday(day_number))
        .copied()
        .flatten()
        .ok_or(StatusCode::BAD_REQUEST)?;
    let existing = planner::entries(&state.pool, household, &day, &day)
        .await
        .map_err(internal_error)?;
    let current = existing.first().and_then(|entry| entry.recipe_id);
    let ids: Vec<i64> = sqlx::query_scalar(
        "SELECT recipe_id FROM recipe_categories WHERE category_id = ? ORDER BY recipe_id",
    )
    .bind(category)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;
    let mut choices: Vec<_> = ids.into_iter().filter(|id| Some(*id) != current).collect();
    if choices.is_empty() {
        choices = existing
            .first()
            .and_then(|entry| entry.recipe_id)
            .into_iter()
            .collect();
    }
    if choices.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }
    let mut rng = fastrand::Rng::with_seed(random_seed());
    let id = choices
        .get(rng.usize(..choices.len()))
        .copied()
        .ok_or(StatusCode::NOT_FOUND)?;
    planner::set_entry(
        &state.pool,
        household,
        &PlanEntry {
            date: day.clone(),
            recipe_id: Some(id),
            servings: existing
                .first()
                .map_or(settings.servings, |entry| entry.servings),
        },
    )
    .await
    .map_err(internal_error)?;
    redirect_week(&day)
}

async fn recipe(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
    Path(day): Path<String>,
    Form(form): Form<RecipeForm>,
) -> Result<Redirect, StatusCode> {
    parse_iso_date(&day).ok_or(StatusCode::BAD_REQUEST)?;
    let household = household(&state, &user).await?;
    let recipe_id = if form.recipe == "rest" {
        None
    } else {
        Some(
            form.recipe
                .parse::<i64>()
                .map_err(|_error| StatusCode::BAD_REQUEST)?,
        )
    };
    if let Some(id) = recipe_id {
        let row: Option<i64> = sqlx::query_scalar("SELECT id FROM recipes WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?;
        if row.is_none() {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    let settings = planner::settings(&state.pool, household)
        .await
        .map_err(internal_error)?;
    let current = planner::entries(&state.pool, household, &day, &day)
        .await
        .map_err(internal_error)?;
    planner::set_entry(
        &state.pool,
        household,
        &PlanEntry {
            date: day.clone(),
            recipe_id,
            servings: current
                .first()
                .map_or(settings.servings, |entry| entry.servings),
        },
    )
    .await
    .map_err(internal_error)?;
    redirect_week(&day)
}

async fn servings(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
    Path(day): Path<String>,
    Form(form): Form<ServingsForm>,
) -> Result<Redirect, StatusCode> {
    parse_iso_date(&day).ok_or(StatusCode::BAD_REQUEST)?;
    let count = form
        .servings
        .parse::<i64>()
        .map_err(|_error| StatusCode::BAD_REQUEST)?;
    if !(1..=99).contains(&count) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let household = household(&state, &user).await?;
    let mut entries = planner::entries(&state.pool, household, &day, &day)
        .await
        .map_err(internal_error)?;
    let mut entry = entries.pop().ok_or(StatusCode::NOT_FOUND)?;
    entry.servings = count;
    planner::set_entry(&state.pool, household, &entry)
        .await
        .map_err(internal_error)?;
    redirect_week(&day)
}

async fn settings(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
    Form(form): Form<SettingsForm>,
) -> Result<Redirect, StatusCode> {
    let lookback_weeks = form
        .lookback_weeks
        .parse::<i64>()
        .map_err(|_error| StatusCode::BAD_REQUEST)?;
    let servings = form
        .servings
        .parse::<i64>()
        .map_err(|_error| StatusCode::BAD_REQUEST)?;
    if !(0..=52).contains(&lookback_weeks) || !(1..=99).contains(&servings) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let weekdays = [
        form.monday,
        form.tuesday,
        form.wednesday,
        form.thursday,
        form.friday,
        form.saturday,
        form.sunday,
    ]
    .map(|value| {
        if value == "rest" {
            Ok(None)
        } else {
            value
                .parse::<i64>()
                .map(Some)
                .map_err(|_error| StatusCode::BAD_REQUEST)
        }
    })
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;
    let weekdays: [Option<i64>; 7] = weekdays
        .try_into()
        .map_err(|_values| StatusCode::BAD_REQUEST)?;
    for category in weekdays.iter().flatten() {
        let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM categories WHERE id = ?")
            .bind(category)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal_error)?;
        if exists.is_none() {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    let household = household(&state, &user).await?;
    planner::save_settings(
        &state.pool,
        household,
        &PlannerSettings {
            lookback_weeks,
            servings,
            weekdays,
        },
    )
    .await
    .map_err(internal_error)?;
    Ok(Redirect::to("/plan"))
}

fn date(day: i64) -> Result<String, StatusCode> {
    format_iso_date(day).ok_or(StatusCode::BAD_REQUEST)
}
fn weekday(day: i64) -> usize {
    usize::try_from((day + 3).rem_euclid(7)).unwrap_or(0)
}

fn random_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| {
            u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
        })
}
