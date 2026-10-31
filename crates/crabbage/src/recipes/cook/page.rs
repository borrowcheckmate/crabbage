//! The cook page: the recipe title, the step in view large with its duration, the mark-done toggle,
//! the previous and next links, and one dot per step to jump to any of them.

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use super::links::Place;
use super::query::{current_step, done_steps};
use crate::i18n::{t, t_with};
use crate::notice::{error_notice, message};
use crate::recipes::duration::duration_text;
use crate::recipes::read::get_recipe;
use crate::recipes::servings::shown_servings;
use crate::recipes::types::RecipeDetail;
use crate::route::route_id;

#[component]
pub(crate) fn CookPage() -> impl IntoView {
    let id = route_id();
    let query = use_query_map();
    view! {
        <Suspense fallback=|| ()>
            {move || {
                let id = id();
                let step = query.with(|map| map.get("step"));
                let servings = query.with(|map| map.get("servings"));
                let done = query.with(|map| map.get("done"));
                Suspend::new(async move { cook_view(id, step, servings, done).await })
            }}
        </Suspense>
    }
}

async fn cook_view(
    id: Option<i64>,
    step: Option<String>,
    servings: Option<String>,
    done: Option<String>,
) -> AnyView {
    let Some(id) = id else {
        return error_notice(t("catalog.error.missing"));
    };
    match get_recipe(id).await {
        Ok(recipe) => cook(
            recipe,
            step.as_deref(),
            servings.as_deref(),
            done.as_deref(),
        ),
        Err(error) => error_notice(&message(&error)),
    }
}

fn cook(
    recipe: RecipeDetail,
    step: Option<&str>,
    requested: Option<&str>,
    done: Option<&str>,
) -> AnyView {
    let RecipeDetail {
        id,
        title,
        servings,
        steps,
        ..
    } = recipe;
    let total = steps.len();
    let current = current_step(step, total);
    let place = Place {
        id,
        servings: requested.map(|_| shown_servings(requested, servings)),
        done: done_steps(done, total),
    };
    let Some(row) = steps.get(current - 1) else {
        return no_steps(title, &place);
    };
    let previous = (current > 1).then(|| place.step(current - 1));
    let next = (current < total).then(|| place.step(current + 1));
    let forward_href = next.clone().unwrap_or_else(|| place.finish());
    let forward_label = if next.is_some() {
        t("cook.next")
    } else {
        t("cook.finish")
    };
    let is_done = place.is_done(current);
    let toggle_label = if is_done {
        t("cook.mark_not_done")
    } else {
        t("cook.mark_done")
    };
    let content = view! {
        {is_done.then(|| view! { <p class="cook-done">{t("cook.done.mark")}</p> })}
        <p class="cook-step">{row.text.clone()}</p>
        {row.duration_seconds.map(|seconds| view! { <p>{duration_text(seconds)}</p> })}
        <a class="button secondary" href=place.toggle(current) aria-pressed=is_done.to_string()>{toggle_label}</a>
        <div class="cook-nav">
            {previous.clone().map(|href| view! { <a class="button secondary" href=href>{t("cook.previous")}</a> })}
            <a class="button" href=forward_href>{forward_label}</a>
        </div>
    }
    .into_any();
    view! {
        <h1 class="cook-title">{title}</h1>
        <p>{t_with("cook.progress", &[("step", &current.to_string()), ("total", &total.to_string())])}</p>
        <progress value=current.to_string() max=total.to_string() aria-label=t("cook.progress.label")></progress>
        {card(previous, next, content)}
        {dots(&place, total, current)}
    }
    .into_any()
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
        <h1 class="cook-title">{title}</h1>
        <p>{t("cook.no_steps")}</p>
        <a class="button secondary" href=place.finish()>{t("cook.back")}</a>
    }
    .into_any()
}

#[cfg(feature = "hydrate")]
fn card(previous: Option<String>, next: Option<String>, content: AnyView) -> AnyView {
    super::swipe::swipe_card(previous, next, content)
}

#[cfg(not(feature = "hydrate"))]
fn card(previous: Option<String>, next: Option<String>, content: AnyView) -> AnyView {
    view! { <div class="cook-card" data-prev=previous data-next=next>{content}</div> }.into_any()
}
