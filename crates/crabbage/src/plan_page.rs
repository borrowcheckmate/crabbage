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

use crate::i18n::t;

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
    let requested = parse_iso_date(week).unwrap_or_else(|| i64::try_from(today).unwrap_or(0));
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
    let names: Vec<(i64, String)> =
        sqlx::query_as("SELECT id, title FROM recipes ORDER BY title COLLATE NOCASE, id")
            .fetch_all(&pool)
            .await
            .map_err(|error| ServerFnError::new(error.to_string()))?;
    let days = plan_days(start, &entries, settings.servings, &names);
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
    names: &[(i64, String)],
) -> Vec<PlanDay> {
    use crabbage_domain::dates::format_iso_date;
    (0_i64..7)
        .map(|offset| {
            let date = format_iso_date(start + offset).unwrap_or_default();
            let entry = entries.iter().find(|entry| entry.date == date);
            let recipe_id = entry.and_then(|entry| entry.recipe_id);
            PlanDay {
                weekday: usize::try_from(offset).unwrap_or(0),
                date,
                recipe_id,
                is_rest: entry.is_some() && recipe_id.is_none(),
                recipe: recipe_id.and_then(|id| {
                    names
                        .iter()
                        .find(|(recipe_id, _)| *recipe_id == id)
                        .map(|(_, name)| name.clone())
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
    let resource = Resource::new(week, plan_week);
    view! {
        <Suspense fallback=|| ()>
            {move || Suspend::new(async move {
                let Ok(data) = resource.await else { return ().into_any(); };
                let start = crabbage_domain::dates::parse_iso_date(&data.week).unwrap_or(0);
                let prev = crabbage_domain::dates::format_iso_date(start - 7).unwrap_or_default();
                let next = crabbage_domain::dates::format_iso_date(start + 7).unwrap_or_default();
                view! {
                    <h1>{t("plan.heading")}</h1>
                    <nav class="plan-week"><a href=format!("/plan?week={prev}")>{t("plan.previous")}</a><span>{data.week.clone()}</span><a href=format!("/plan?week={next}")>{t("plan.next")}</a></nav>
                    <form method="post" action="/plan/generate"><input type="hidden" name="week" value=data.week.clone()/><button>{t("plan.generate")}</button></form>
                    <For each=move || data.days.clone() key=|day| day.date.clone() let(day)>
                        <section class="plan-day"><h2>{t(weekday_key(day.weekday))} {day.date.clone()}</h2>
                            {match (day.recipe_id, day.recipe.clone()) {
                                (Some(id), Some(name)) => view! { <p><a href=format!("/recipes/{id}?servings={}", day.servings)>{name}</a> <a href=format!("/recipes/{id}/cook?servings={}", day.servings)>{t("plan.cook")}</a></p> }.into_any(),
                                (None, _) => view! { <p>{if day.is_rest { t("plan.rest") } else { t("plan.empty") }}</p> }.into_any(),
                                _ => view! { <p>{t("plan.empty")}</p> }.into_any()
                            }}
                            <form method="post" action=format!("/plan/{}/reroll", day.date)><button class="secondary">{t("plan.reroll")}</button></form>
                            <form method="post" action=format!("/plan/{}/recipe", day.date)><select name="recipe"><option value="rest">{t("plan.rest")}</option>{data.recipes.iter().map(|recipe| view! { <option value=recipe.0.to_string() selected=day.recipe_id == Some(recipe.0)>{recipe.1.clone()}</option> }).collect_view()}</select><button>{t("plan.save")}</button></form>
                            <form method="post" action=format!("/plan/{}/servings", day.date)><label>{t("recipe.servings")}<input name="servings" type="number" min="1" max="99" value=day.servings/></label><button>{t("plan.save")}</button></form>
                        </section>
                    </For>
                    <h2>{t("plan.settings")}</h2>
                    <form class="plan-settings" method="post" action="/plan/settings">
                        <For each=weekday_labels key=|item| *item let(item)>
                            <label>{t(weekday_key(item))}<select name=weekday_field(item)><option value="rest">{t("plan.rest")}</option>{data.categories.iter().map(|category| view! { <option value=category.0.to_string() selected=data.weekdays.get(item).copied().flatten() == Some(category.0)>{category.1.clone()}</option> }).collect_view()}</select></label>
                        </For>
                        <label>{t("plan.lookback")}<input name="lookback_weeks" type="number" min="0" max="52" value=data.lookback/></label>
                        <label>{t("recipe.servings")}<input name="servings" type="number" min="1" max="99" value=data.servings/></label>
                        <button>{t("plan.save")}</button>
                    </form>
                }.into_any()
            })}
        </Suspense>
    }
}

fn weekday_labels() -> Vec<usize> {
    (0..7).collect()
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
    .copied()
    .unwrap_or("plan.monday")
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
    .copied()
    .unwrap_or("monday")
}
