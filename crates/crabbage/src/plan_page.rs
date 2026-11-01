#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces the server function body with a stub that does not await"
    )
)]

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};

use crate::i18n::{t, t_with};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct PlanData {
    week: String,
    days: Vec<PlanDay>,
    categories: Vec<(i64, String)>,
    recipes: Vec<(i64, String)>,
    weekdays: Vec<Option<i64>>,
    servings: i64,
    lookback: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct PlanDay {
    date: String,
    weekday: usize,
    recipe_id: Option<i64>,
    recipe: Option<String>,
    photo: Option<String>,
    is_rest: bool,
    servings: i64,
}

#[server(prefix = "/api", endpoint = "plan_week")]
async fn plan_week(week: String) -> Result<PlanData, ServerFnError> {
    load_plan_week(&week).await
}

#[cfg(feature = "ssr")]
async fn load_plan_week(week: &str) -> Result<PlanData, ServerFnError> {
    use crabbage_domain::dates::{format_iso_date, parse_iso_date};
    use crabbage_store::{catalog::categories, households::active_household, planner};
    let (pool, user) = crate::access::signed_in().await?;
    let today = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| ServerFnError::new(error.to_string()))?
        .as_secs()
        / 86400;
    let today = i64::try_from(today).map_err(|error| ServerFnError::new(error.to_string()))?;
    let requested = parse_iso_date(week).unwrap_or(today);
    let start = requested - (requested + 3).rem_euclid(7);
    let start_text = format_iso_date(start).ok_or_else(|| ServerFnError::new("Invalid week"))?;
    let end_text = format_iso_date(start + 6).ok_or_else(|| ServerFnError::new("Invalid week"))?;
    let household = active_household(&pool, user)
        .await
        .map_err(|error| crate::access::store_failure(&error))?
        .ok_or_else(|| ServerFnError::new("No active household"))?;
    let settings = planner::settings(&pool, household)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    let entries = planner::entries(&pool, household, &start_text, &end_text)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    let recipe_rows: Vec<(i64, String, Option<String>)> =
        sqlx::query_as("SELECT id, title, image FROM recipes ORDER BY title COLLATE NOCASE, id")
            .fetch_all(&pool)
            .await
            .map_err(|error| ServerFnError::new(error.to_string()))?;
    let names = recipe_rows
        .iter()
        .map(|(id, title, _)| (*id, title.clone()))
        .collect::<Vec<_>>();
    let days = plan_days(start, &entries, settings.servings, &recipe_rows);
    let categories = categories::list(&pool)
        .await
        .map_err(|error| crate::access::store_failure(&error))?
        .into_iter()
        .map(|category| (category.id, category.name))
        .collect();
    Ok(PlanData {
        week: start_text,
        days,
        categories,
        recipes: names,
        weekdays: settings.weekdays.to_vec(),
        servings: settings.servings,
        lookback: settings.lookback_weeks,
    })
}

#[cfg(feature = "ssr")]
fn plan_days(
    start: i64,
    entries: &[crabbage_store::planner::PlanEntry],
    default_servings: i64,
    names: &[(i64, String, Option<String>)],
) -> Vec<PlanDay> {
    use crabbage_domain::dates::format_iso_date;
    (0_i64..7)
        .map(|offset| {
            let date = format_iso_date(start + offset).unwrap_or_default();
            let entry = entries.iter().find(|entry| entry.date == date);
            let recipe_id = entry.and_then(|entry| entry.recipe_id);
            PlanDay {
                weekday: usize::try_from(offset).unwrap_or_default(),
                date,
                recipe_id,
                photo: recipe_id.and_then(|id| {
                    names
                        .iter()
                        .find(|(recipe_id, _, _)| *recipe_id == id)
                        .and_then(|(_, _, photo)| photo.clone())
                }),
                is_rest: entry.is_some() && recipe_id.is_none(),
                recipe: recipe_id.and_then(|id| {
                    names
                        .iter()
                        .find(|(recipe_id, _, _)| *recipe_id == id)
                        .map(|(_, name, _)| name.clone())
                }),
                servings: entry.map_or(default_servings, |entry| entry.servings),
            }
        })
        .collect()
}

#[component]
pub(crate) fn PlanPage() -> impl IntoView {
    let query = use_query_map();
    let week = move || query.get().get("week").unwrap_or_default();
    let selected = move || query.get().get("day").unwrap_or_default();
    let resource = Resource::new(week, plan_week);
    view! {
        <Suspense fallback=|| view! { <p class="plan-loading" role="status">{t("plan.loading")}</p> }>
            {move || Suspend::new(async move {
                resource.await.map_or_else(
                    |_| plan_error().into_any(),
                    |data| {
                        let chosen = selected();
                        plan_view(data, &chosen).into_any()
                    },
                )
            })}
        </Suspense>
    }
}

fn plan_error() -> impl IntoView {
    view! { <section class="plan-error"><h1>{t("plan.heading")}</h1><p class="notice error">{t("plan.failed")}</p></section> }
}

fn plan_view(data: PlanData, chosen: &str) -> impl IntoView {
    let data = StoredValue::new(data);
    let plan = data.get_value();
    let week = plan.week;
    let start = crabbage_domain::dates::parse_iso_date(&week).unwrap_or_default();
    let prev = crabbage_domain::dates::format_iso_date(start - 7).unwrap_or_default();
    let next = crabbage_domain::dates::format_iso_date(start + 7).unwrap_or_default();
    let selected_day = plan
        .days
        .iter()
        .find(|day| day.date == chosen)
        .or_else(|| plan.days.first())
        .cloned();
    let selected_date = selected_day
        .as_ref()
        .map_or_else(String::new, |day| day.date.clone());
    view! {
        <div class="plan-heading"><h1>{t("plan.heading")}</h1><div class="plan-actions">
            <form method="post" action="/plan/generate"><input type="hidden" name="week" value=week.clone()/><button>{t("plan.generate")}</button></form>
        </div></div>
        <nav class="plan-week" aria-label=t("plan.week_navigation")><a href=format!("/plan?week={prev}") aria-label=t("plan.previous")><svg aria-hidden="true" viewBox="0 0 24 24"><path d="m15 18-6-6 6-6"/></svg></a><span>{week_range(&week)}</span><a href=format!("/plan?week={next}") aria-label=t("plan.next")><svg aria-hidden="true" viewBox="0 0 24 24"><path d="m9 18 6-6-6-6"/></svg></a></nav>
        <div class="plan-layout"><PlanAgenda data=data.get_value() selected_date=selected_date/>
            {selected_day.map(|day| view! { <PlanEditor day=day recipes=data.get_value().recipes week=week.clone()/> })}
        </div>
    }
}

#[component]
fn PlanAgenda(data: PlanData, selected_date: String) -> impl IntoView {
    let week = data.week.clone();
    let days = StoredValue::new(data.days.clone());
    let settings = plan_settings(data);
    view! {
        <div class="plan-agenda" aria-label=t("plan.agenda")>
            <For each=move || days.get_value() key=|day| day.date.clone() let(day)>
                <a class="plan-day" aria-current=(selected_date == day.date).then_some("date") href=format!("/plan?week={}&day={}", week.clone(), day.date)>
                    <span class="plan-day-date"><b>{t(weekday_short_key(day.weekday))}</b><time datetime=day.date.clone()>{day_number(&day.date)}</time></span>
                    <span class="plan-day-summary"><strong>{day_title(&day)}</strong>{(!day.is_rest).then(|| view! { <small>{day_subtitle(&day)}</small> })}</span>
                    <svg aria-hidden="true" viewBox="0 0 24 24"><path d="m9 18 6-6-6-6"/></svg>
                </a>
            </For>
            {settings}
        </div>
    }
}

#[component]
fn PlanEditor(day: PlanDay, recipes: Vec<(i64, String)>, week: String) -> impl IntoView {
    let selected_photo = day.photo.clone();
    let date = day.date.clone();
    let recipe_action = format!("/plan/{date}/recipe");
    let servings_action = format!("/plan/{date}/servings");
    let reroll_action = format!("/plan/{date}/reroll");
    view! {
        <section class="plan-editor"><p class="plan-date-label">{format!("{} {} {}", t(weekday_short_key(day.weekday)), t("plan.separator"), t("plan.dinner"))}</p><h2>{day_title(&day)}</h2>
            {selected_photo.map_or_else(|| view! { <div class="plan-editor-photo plan-photo-fallback" aria-hidden="true"><svg viewBox="0 0 24 24"><path d="M3 10h18l-2 10H5L3 10Zm3 0a6 6 0 0 1 12 0"/></svg></div> }.into_any(), |photo| view! { <img class="plan-editor-photo" src=format!("/images/{photo}") alt=""/> }.into_any())}
            <div class="plan-servings"><span>{t("recipe.servings")}</span><form method="post" action=servings_action.clone()><input type="hidden" name="servings" value=(day.servings - 1).max(1)/><button aria-label=t("plan.decrease_servings") disabled={day.servings <= 1}>-</button></form><strong>{day.servings}</strong><form method="post" action=servings_action><input type="hidden" name="servings" value=(day.servings + 1).min(99)/><button aria-label=t("plan.increase_servings") disabled={day.servings >= 99}>+</button></form></div>
            <form method="post" action=recipe_action class="plan-recipe-picker"><label>{t("plan.recipe")}<select name="recipe"><option value="rest">{t("plan.rest")}</option>{recipes.iter().map(|recipe| view! { <option value=recipe.0.to_string() selected=day.recipe_id == Some(recipe.0)>{recipe.1.clone()}</option> }).collect_view()}</select></label><button>{t("plan.change_recipe")}</button></form>
            <form class="plan-reroll" method="post" action=reroll_action><button class="secondary">{t("plan.reroll")}</button></form>
            <form class="plan-editor-shopping" method="post" action="/plan/shopping"><input type="hidden" name="week" value=week/><button class="secondary">{t("plan.add_shopping")}</button></form>
            {day.recipe_id.map(|id| view! { <div class="plan-recipe-links"><a href=format!("/recipes/{id}?servings={}", day.servings)>{t("plan.view_recipe")}</a><a href=format!("/recipes/{id}/cook?servings={}", day.servings)>{t("plan.cook")}</a></div> })}
        </section>
    }
}

fn weekday_labels() -> Vec<usize> {
    (0..7).collect()
}

fn plan_settings(data: PlanData) -> impl IntoView {
    let settings = StoredValue::new(data);
    view! {
        <details class="plan-settings"><summary>{t("plan.settings")}</summary>
            <form method="post" action="/plan/settings">
                <For each=weekday_labels key=|item| *item let(item)>
                    <label>{t(weekday_key(item))}<select name=weekday_field(item)><option value="rest">{t("plan.rest")}</option>{settings.get_value().categories.iter().map(|category| view! { <option value=category.0.to_string() selected=settings.get_value().weekdays.get(item).copied().flatten() == Some(category.0)>{category.1.clone()}</option> }).collect_view()}</select></label>
                </For>
                <label>{t("plan.lookback")}<input name="lookback_weeks" type="number" min="0" max="52" value=settings.get_value().lookback/></label>
                <label>{t("recipe.servings")}<input name="servings" type="number" min="1" max="99" value=settings.get_value().servings/></label>
                <button>{t("plan.save")}</button>
            </form>
        </details>
    }
}

fn weekday_key(index: usize) -> &'static str {
    [
        "plan.monday",
        "plan.tuesday",
        "plan.wednesday",
        "plan.thursday",
        "plan.friday",
        "plan.saturday",
        "plan.sunday",
    ]
    .get(index)
    .map_or("plan.monday", |key| *key)
}

fn weekday_short_key(index: usize) -> &'static str {
    [
        "plan.monday.short",
        "plan.tuesday.short",
        "plan.wednesday.short",
        "plan.thursday.short",
        "plan.friday.short",
        "plan.saturday.short",
        "plan.sunday.short",
    ]
    .get(index)
    .map_or("plan.monday.short", |key| *key)
}

fn day_number(date: &str) -> String {
    date.split('-')
        .nth(2)
        .and_then(|part| part.parse::<u8>().ok())
        .map_or_else(String::new, |day| day.to_string())
}

fn month_name(date: &str) -> Option<String> {
    let month = date.split('-').nth(1)?.parse::<u8>().ok()?;
    Some(t(month_key(month)).to_owned())
}

fn month_key(month: u8) -> &'static str {
    [
        "date.month.1",
        "date.month.2",
        "date.month.3",
        "date.month.4",
        "date.month.5",
        "date.month.6",
        "date.month.7",
        "date.month.8",
        "date.month.9",
        "date.month.10",
        "date.month.11",
        "date.month.12",
    ]
    .get(usize::from(month.saturating_sub(1)))
    .map_or("date.month.1", |key| *key)
}

fn week_range(start: &str) -> String {
    let end = crabbage_domain::dates::parse_iso_date(start)
        .and_then(|day| crabbage_domain::dates::format_iso_date(day + 6));
    match (month_name(start), end.as_deref().and_then(month_name)) {
        (Some(start_month), Some(end_month)) if start_month == end_month => {
            format!(
                "{} {} – {}",
                day_number(start),
                start_month,
                end.as_deref().map_or_else(String::new, day_number)
            )
        }
        (Some(start_month), Some(end_month)) => {
            format!(
                "{} {} – {} {}",
                day_number(start),
                start_month,
                end.as_deref().map_or_else(String::new, day_number),
                end_month
            )
        }
        _ => start.to_owned(),
    }
}
fn weekday_field(index: usize) -> &'static str {
    [
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "friday",
        "saturday",
        "sunday",
    ]
    .get(index)
    .map_or("monday", |field| *field)
}

fn day_title(day: &PlanDay) -> String {
    match day.recipe.clone() {
        Some(recipe) => recipe,
        None if day.is_rest => t("plan.rest").to_owned(),
        None => t("plan.empty").to_owned(),
    }
}

fn day_subtitle(day: &PlanDay) -> String {
    match day.recipe.as_ref() {
        Some(_) => t_with(
            "recipe.servings.many",
            &[("count", &day.servings.to_string())],
        ),
        None if day.is_rest => String::new(),
        None => t("plan.no_recipe").to_owned(),
    }
}
