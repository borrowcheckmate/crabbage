//! The cook page: the recipe title, the step in view large with its duration, the mark-done toggle,
//! the previous and next links, and one dot per step to jump to any of them.

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use super::links::Place;
use super::query::{current_step, done_steps};
use super::timers::{Timer, step_durations};
use super::tray::{TimerTray, start_button};
use crate::i18n::{messages, t, t_with};
use crate::notice::{error_notice, message};
use crate::recipes::line_text::line_text;
use crate::recipes::read::get_recipe;
use crate::recipes::servings::{scale_lines, shown_servings};
use crate::recipes::types::{LineRow, RecipeDetail, StepRow};
use crate::route::route_id;

type CookPageData = (
    Option<i64>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<Result<RecipeDetail, ServerFnError>>,
);

#[component]
pub(crate) fn CookPage() -> impl IntoView {
    let id = route_id();
    let query = use_query_map();
    let timers = RwSignal::new(Vec::<Timer>::new());
    let now = RwSignal::new(0.0_f64);
    let data = Resource::new(
        move || {
            (
                id(),
                query.with(|map| (map.get("step"), map.get("servings"), map.get("done"))),
            )
        },
        |(id, (step, servings, done))| async move {
            let recipe = match id {
                Some(id) => Some(get_recipe(id).await),
                None => None,
            };
            (id, step, servings, done, recipe)
        },
    );
    #[cfg(feature = "hydrate")]
    super::wake_client::attach();
    view! {
        <Suspense fallback=|| ()>
            {move || {
                Suspend::new(async move { cook_view(data.await, timers) })
            }}
        </Suspense>
        <TimerTray timers now/>
    }
}

fn cook_view(
    (id, step, servings, done, recipe): CookPageData,
    timers: RwSignal<Vec<Timer>>,
) -> AnyView {
    if id.is_none() {
        return error_notice(t("catalog.error.missing"));
    }
    match recipe {
        Some(Ok(recipe)) => cook(
            recipe,
            step.as_deref(),
            servings.as_deref(),
            done.as_deref(),
            timers,
        ),
        Some(Err(error)) => error_notice(&message(&error)),
        None => error_notice(t("catalog.error.missing")),
    }
}

fn cook(
    recipe: RecipeDetail,
    step: Option<&str>,
    requested: Option<&str>,
    done: Option<&str>,
    timers: RwSignal<Vec<Timer>>,
) -> AnyView {
    let RecipeDetail {
        id,
        title,
        servings,
        lines,
        steps,
        ..
    } = recipe;
    let total = steps.len();
    let current = current_step(step, total);
    let shown = shown_servings(requested, servings);
    let place = Place {
        id,
        servings: requested.map(|_| shown),
        done: done_steps(done, total),
    };
    let Some(row) = steps.get(current - 1) else {
        return no_steps(title, &place);
    };
    let previous = (current > 1).then(|| place.step(current - 1));
    let next = (current < total).then(|| place.step(current + 1));
    let is_done = place.is_done(current);
    let content = step_content(
        row,
        current,
        &place,
        previous.clone(),
        next.as_deref(),
        timers,
        is_done,
    );
    view! {
        <section class="cook-mode" aria-label=t("cook.mode.label")>
            <a class="cook-return" href=format!("/recipes/{id}")>
                <svg aria-hidden="true" viewBox="0 0 24 24"><path d="m15 18-6-6 6-6M9 12h12"/></svg>
                {t("cook.back")}
            </a>
            <p class="cook-recipe-name">{title}</p>
            <h1 class="cook-title">{t("cook.heading")}</h1>
            <div class="cook-progress" aria-live="polite">
                <p>{t_with("cook.progress", &[("step", &current.to_string()), ("total", &total.to_string())])}</p>
                <progress value=current.to_string() max=total.to_string() aria-label=t("cook.progress.label")></progress>
            </div>
            <div class="cook-layout">
                {card(previous, next, content)}
                {needed(row, &lines, servings, shown)}
            </div>
            <a class="cook-toggle" href=place.toggle(current) aria-pressed=is_done.to_string()>
                {done_label(is_done)}
            </a>
            {dots(&place, total, current)}
        </section>
    }
    .into_any()
}

fn needed_text(line: &LineRow) -> String {
    let ingredient = line.ingredient.as_deref().unwrap_or_default();
    if line.amount.is_none() {
        return line_text(line, messages());
    }
    let mut quantity_line = line.clone();
    quantity_line.ingredient = None;
    quantity_line.note = None;
    let quantity = line_text(&quantity_line, messages());
    let note = line.note.as_deref().unwrap_or_default();
    let label = if ingredient.is_empty() {
        quantity
    } else {
        format!("{ingredient} · {quantity}")
    };
    if note.is_empty() {
        label
    } else {
        format!("{label}, {note}")
    }
}

fn step_content(
    row: &StepRow,
    current: usize,
    place: &Place,
    previous: Option<String>,
    next: Option<&str>,
    timers: RwSignal<Vec<Timer>>,
    is_done: bool,
) -> AnyView {
    let forward_href = next.map_or_else(|| place.finish(), str::to_owned);
    let forward_label = if next.is_some() {
        t("cook.next_step")
    } else {
        t("cook.finish")
    };
    view! {
        {is_done.then(|| view! { <p class="cook-done">{t("cook.done.mark")}</p> })}
        <p class="cook-step">{row.text.clone()}</p>
        {timer_starts(row, current, timers)}
        <div class="cook-nav">
            <a class="button secondary" href=previous.unwrap_or_else(|| place.finish())>{t("cook.previous")}</a>
            <a class="button" href=forward_href>
                {forward_label}
                {next.map(|_| view! { <svg aria-hidden="true" viewBox="0 0 24 24"><path d="M5 12h14m-6-6 6 6-6 6"/></svg> })}
            </a>
        </div>
    }.into_any()
}

fn done_label(is_done: bool) -> &'static str {
    if is_done {
        t("cook.mark_not_done")
    } else {
        t("cook.mark_done")
    }
}

/// One start button per duration the step names, for step number `step`.
fn timer_starts(row: &StepRow, step: usize, timers: RwSignal<Vec<Timer>>) -> AnyView {
    let buttons = step_durations(&row.text, row.duration_seconds)
        .into_iter()
        .map(|seconds| start_button(timers, step, seconds))
        .collect_view();
    view! { <div class="timer-starts">{buttons}</div> }.into_any()
}

/// The lines the step uses, scaled from the recipe's servings to `shown`, or nothing when it uses none.
fn needed(step: &StepRow, lines: &[LineRow], servings: i64, shown: i64) -> Option<AnyView> {
    let used: Vec<LineRow> = step
        .line_positions
        .iter()
        .filter_map(|&position| lines.get(position).cloned())
        .collect();
    (!used.is_empty()).then(|| {
        let items = scale_lines(&used, servings, shown)
            .iter()
            .map(|line| view! { <li>{needed_text(line)}</li> })
            .collect_view();
        view! {
            <section class="cook-needs">
                <h2>{t_with("cook.needs", &[("servings", &shown.to_string())])}</h2>
                <ul>{items}</ul>
            </section>
        }
        .into_any()
    })
}

fn dots(place: &Place, total: usize, current: usize) -> AnyView {
    let items = (1..=total)
        .map(|number| {
            let label = t_with("cook.step.go", &[("step", &number.to_string())]);
            let class = if place.is_done(number) {
                "cook-dot done"
            } else {
                "cook-dot"
            };
            view! {
                <li>
                    <a href=place.step(number) class=class aria-label=label aria-current=(number == current).then_some("step")>
                        {number.to_string()}
                    </a>
                </li>
            }
        })
        .collect_view();
    view! { <nav aria-label=t("cook.steps.label")><ol class="cook-dots">{items}</ol></nav> }
        .into_any()
}

fn no_steps(title: String, place: &Place) -> AnyView {
    view! {
        <section class="cook-mode" aria-label=t("cook.mode.label")>
            <a class="cook-return" href=place.finish()>
                <svg aria-hidden="true" viewBox="0 0 24 24"><path d="m15 18-6-6 6-6M9 12h12"/></svg>
                {t("cook.back")}
            </a>
            <p class="cook-recipe-name">{title}</p>
            <h1 class="cook-title">{t("cook.heading")}</h1>
            <p>{t("cook.no_steps")}</p>
        </section>
    }
    .into_any()
}

#[cfg(feature = "hydrate")]
fn card(previous: Option<String>, next: Option<String>, content: AnyView) -> AnyView {
    super::swipe::swipe_card(previous, next, content)
}

#[cfg(not(feature = "hydrate"))]
fn card(previous: Option<String>, next: Option<String>, content: AnyView) -> AnyView {
    view! { <div class="cook-instruction" data-prev=previous data-next=next>{content}</div> }
        .into_any()
}
