#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces the server function body with a stub that does not await"
    )
)]

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{t, t_with};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct TodayData {
    date: String,
    weekday: usize,
    household: Option<String>,
    meal: Option<TodayMeal>,
    upcoming: Vec<TodayMeal>,
    shopping_count: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct TodayMeal {
    date: String,
    weekday: usize,
    recipe_id: Option<i64>,
    title: Option<String>,
    photo: Option<String>,
    servings: i64,
    rest: bool,
}

#[server(prefix = "/api", endpoint = "today")]
async fn today() -> Result<TodayData, ServerFnError> {
    let (pool, user) = crate::access::signed_in().await?;
    let day = utc_day_number()?;
    let date = crabbage_domain::dates::format_iso_date(day)
        .ok_or_else(|| ServerFnError::new("Invalid date"))?;
    let household = crabbage_store::households::active_household(&pool, user)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    let Some(household) = household else {
        return Ok(TodayData {
            date,
            weekday: weekday_index(day),
            household: None,
            meal: None,
            upcoming: Vec::new(),
            shopping_count: 0,
        });
    };
    load_today_data(&pool, household, day, date).await
}

#[cfg(feature = "ssr")]
async fn load_today_data(
    pool: &sqlx::SqlitePool,
    household: i64,
    day: i64,
    date: String,
) -> Result<TodayData, ServerFnError> {
    let household_name: String =
        sqlx::query_scalar("SELECT group_name FROM households WHERE id = ?")
            .bind(household)
            .fetch_one(pool)
            .await
            .map_err(|error| ServerFnError::new(error.to_string()))?;
    let dates = (day..=day + 3)
        .map(crabbage_domain::dates::format_iso_date)
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| ServerFnError::new("Invalid date"))?;
    let end_date = dates
        .get(3)
        .ok_or_else(|| ServerFnError::new("Invalid date"))?;
    let meals = upcoming_meals(pool, household, &date, end_date).await?;
    let today_row = sqlx::query_as::<_, (Option<i64>, i64, Option<String>, Option<String>)>(
        "SELECT p.recipe_id, p.servings, r.title, r.image \
         FROM plan_entries p LEFT JOIN recipes r ON r.id = p.recipe_id \
         WHERE p.household_id = ? AND p.date = ?",
    )
    .bind(household)
    .bind(&date)
    .fetch_optional(pool)
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))?;
    let meal = today_row.map(|(recipe_id, servings, title, photo)| TodayMeal {
        date: date.clone(),
        weekday: weekday_index(day),
        recipe_id,
        title,
        photo,
        servings,
        rest: recipe_id.is_none(),
    });
    let shopping_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM shopping_items WHERE household_id = ? AND checked = 0",
    )
    .bind(household)
    .fetch_one(pool)
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))?;
    Ok(TodayData {
        date,
        weekday: weekday_index(day),
        household: Some(household_name),
        meal,
        upcoming: meals,
        shopping_count,
    })
}

#[cfg(feature = "ssr")]
async fn upcoming_meals(
    pool: &sqlx::SqlitePool,
    household: i64,
    date: &str,
    end_date: &str,
) -> Result<Vec<TodayMeal>, ServerFnError> {
    let rows = sqlx::query_as::<_, (String, Option<i64>, i64, Option<String>, Option<String>)>(
        "SELECT p.date, p.recipe_id, p.servings, r.title, r.image \
         FROM plan_entries p LEFT JOIN recipes r ON r.id = p.recipe_id \
         WHERE p.household_id = ? AND p.date > ? AND p.date <= ? ORDER BY p.date",
    )
    .bind(household)
    .bind(date)
    .bind(end_date)
    .fetch_all(pool)
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))?;
    Ok(rows.into_iter().filter_map(meal_from_row).collect())
}

#[cfg(feature = "ssr")]
fn meal_from_row(
    (date, recipe_id, servings, title, photo): (
        String,
        Option<i64>,
        i64,
        Option<String>,
        Option<String>,
    ),
) -> Option<TodayMeal> {
    let day = crabbage_domain::dates::parse_iso_date(&date)?;
    let weekday = usize::try_from((day + 3).rem_euclid(7)).ok()?;
    Some(TodayMeal {
        date,
        weekday,
        recipe_id,
        title,
        photo,
        servings,
        rest: recipe_id.is_none(),
    })
}

#[cfg(feature = "ssr")]
fn utc_day_number() -> Result<i64, ServerFnError> {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| ServerFnError::new(error.to_string()))?
        .as_secs();
    i64::try_from(seconds / 86_400).map_err(|error| ServerFnError::new(error.to_string()))
}

#[cfg(feature = "ssr")]
fn weekday_index(day: i64) -> usize {
    usize::try_from((day + 3).rem_euclid(7)).unwrap_or_default()
}

#[component]
pub(crate) fn TodayPage() -> impl IntoView {
    let resource = Resource::new(|| (), |()| today());
    view! {
        <Suspense fallback=|| view! { <p class="today-loading" role="status">{t("home.loading")}</p> }>
            {move || Suspend::new(async move {
                resource.await.map_or_else(
                    |_| view! { <section class="today-empty"><h1>{t("home.heading")}</h1><p class="notice">{t("home.failed")}</p></section> }.into_any(),
                    |data| today_view(data).into_any(),
                )
            })}
        </Suspense>
    }
}

fn today_view(data: TodayData) -> impl IntoView {
    let has_household = data.household.is_some();
    let no_upcoming = data.upcoming.is_empty();
    let meal = data.meal;
    let upcoming = StoredValue::new(data.upcoming);
    view! {
        <div class="today-heading"><p class="today-date">{format_long_date(&data.date, data.weekday)}</p><h1>{t("home.heading")}</h1>
        </div>
        <Show when=move || has_household fallback=|| view! {
            <section class="today-empty"><p>{t("home.no_household")}</p></section>
        }>
            <div class="today-layout">
                <section class="today-meal">
                    {match meal.clone() {
                        Some(meal) if meal.rest => view! { <div class="today-rest"><p>{t("home.rest")}</p><a class="today-action secondary" href="/plan">{t("home.open_plan")}</a></div> }.into_any(),
                        Some(meal) if meal.title.is_some() => {
                            let id = meal.recipe_id.unwrap_or_default();
                            let title = meal.title.unwrap_or_default();
                            let photo = meal.photo;
                            view! { <article class="today-feature">
                                {photo.map_or_else(
                                    || view! { <div class="today-photo-fallback" aria-hidden="true"><svg viewBox="0 0 24 24"><path d="M3 10h18l-2 10H5L3 10Z"/><path d="M6 10a6 6 0 0 1 12 0M8 4l1 2m7-2-1 2"/></svg></div> }.into_any(),
                                    |name| view! { <img src=format!("/images/{name}") alt=""/> }.into_any(),
                                )}
                                <div class="today-feature-copy"><p class="today-feature-eyebrow">{t("home.today_meal")}</p><h2>{title}</h2>
                                <p class="today-feature-meta"><svg aria-hidden="true" viewBox="0 0 24 24"><path d="M16 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2m6-10a4 4 0 1 0 0-8 4 4 0 0 0 0 8Zm8-1a4 4 0 0 1 0 7m4 4v-2a4 4 0 0 0-3-3.87"/></svg>{servings_text(meal.servings)}</p>
                                <a class="today-action" href=format!("/recipes/{id}/cook?servings={}", meal.servings)><svg aria-hidden="true" viewBox="0 0 24 24"><path d="M3 10h18l-2 10H5L3 10Zm3 0a6 6 0 0 1 12 0M8 4l1 2m7-2-1 2M12 14v3"/></svg>{t("recipe.button.start_cooking")}<svg aria-hidden="true" viewBox="0 0 24 24"><path d="M7 17 17 7M7 7h10v10"/></svg></a></div>
                            </article> }.into_any()
                        }
                        _ => view! { <div class="today-empty"><p>{t("home.empty")}</p><a class="today-action secondary" href="/plan">{t("home.open_plan")}</a></div> }.into_any(),
                    }}
                </section>
                <div class="today-side">
                    <section class="today-upcoming"><div class="today-section-heading"><h2>{t("home.coming_up")}</h2><a href="/plan">{t("home.see_week")}</a></div>
                        <For each=move || upcoming.get_value() key=|item| item.date.clone() let(item)>
                            <div class="today-next"><span>{t(weekday_short_key(item.weekday))}</span><div><h3>{match item.title.clone() { Some(title) => title, None if item.rest => t("home.rest").to_owned(), None => t("home.empty_short").to_owned() }}</h3>{item.title.is_some().then(|| view! { <p>{servings_text(item.servings)}</p> })}</div></div>
                        </For>
                        {no_upcoming.then(|| view! { <p class="today-muted">{t("home.no_upcoming")}</p> })}
                    </section>
                    <a class="today-shopping" href="/shopping"><svg class="today-basket" aria-hidden="true" viewBox="0 0 24 24"><path d="M3 10h18l-2 10H5L3 10Zm4 0 5-7 5 7m-8 4v3m6-3v3"/></svg><span>{t("home.shopping")}<strong>{shopping_count_text(data.shopping_count)}</strong></span><svg class="today-arrow" aria-hidden="true" viewBox="0 0 24 24"><path d="M5 12h14m-6-6 6 6-6 6"/></svg></a>
                </div>
            </div>
        </Show>
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

fn format_long_date(date: &str, weekday: usize) -> String {
    let mut parts = date.split('-');
    let _year = parts.next();
    let month = parts.next().and_then(|value| value.parse::<u8>().ok());
    let day = parts.next().and_then(|value| value.parse::<u8>().ok());
    match (month, day) {
        (Some(month), Some(day)) => format!(
            "{}, {} {}",
            t(weekday_key(weekday)),
            day,
            t(month_key(month))
        ),
        _ => t(weekday_key(weekday)).to_owned(),
    }
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

fn servings_text(count: i64) -> String {
    let key = if count == 1 {
        "recipe.servings.one"
    } else {
        "recipe.servings.many"
    };
    t_with(key, &[("count", &count.to_string())])
}

fn shopping_count_text(count: i64) -> String {
    let key = if count == 1 {
        "home.shopping_count.one"
    } else {
        "home.shopping_count.many"
    };
    t_with(key, &[("count", &count.to_string())])
}
