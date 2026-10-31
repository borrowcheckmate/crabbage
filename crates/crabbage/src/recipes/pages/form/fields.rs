//! The fields of the recipe form: the title, the servings, the ingredient rows and the step rows,
//! each row with a button to remove it.

use crabbage_domain::quantity::Unit;
use leptos::prelude::*;
use leptos::prelude::{event_target_checked, event_target_value};

use super::draft::{Draft, LineSignals, StepSignals};
use crate::catalog::CategoryRow;
use crate::i18n::{messages, t};
use crate::recipes::types::{LineDraft, StepDraft};
use crate::recipes::units::unit_text;

/// The fields of the recipe form. The names fill the suggestions of the ingredient fields. A row
/// whose ingredient is in `unresolved` carries the class `unresolved` while it keeps that name.
#[component]
pub(crate) fn DraftFields(
    draft: Draft,
    names: Vec<String>,
    #[prop(optional)] unresolved: Vec<String>,
) -> AnyView {
    view! {
        <EssentialsFields draft/>
        <IngredientFields draft names unresolved/>
        <MethodFields draft/>
    }
    .into_any()
}

#[component]
fn EssentialsFields(draft: Draft) -> AnyView {
    view! {
        <section class="recipe-section recipe-essentials">
            <h2>{t("recipe.section.essentials")}</h2>
            {title_field(draft)}
            {servings_field(draft)}
        </section>
    }
    .into_any()
}

#[component]
fn IngredientFields(draft: Draft, names: Vec<String>, unresolved: Vec<String>) -> AnyView {
    let suggestions = names
        .into_iter()
        .map(|name| view! { <option value=name/> })
        .collect_view();
    view! {
        <section class="recipe-section">
            <h2>{t("recipe.lines.heading")}</h2>
            <div class="rows">
                <For each=move || draft.lines.get() key=|row| row.key children=move |row| line_row(draft, row, unresolved.clone())/>
            </div>
            <button type="button" class="secondary" on:click=move |_| draft.add_line(LineDraft::default())>
                {t("recipe.button.add_line")}
            </button>
        </section>
        <datalist id="ingredient-names">{suggestions}</datalist>
    }
    .into_any()
}

#[component]
fn MethodFields(draft: Draft) -> AnyView {
    view! {
        <section class="recipe-section">
            <h2>{t("recipe.steps.heading")}</h2>
            <div class="rows">
                <For each=move || draft.steps.get() key=|row| row.key children=move |row| step_row(draft, row)/>
            </div>
            <button type="button" class="secondary" on:click=move |_| draft.add_step(StepDraft::default())>
                {t("recipe.button.add_step")}
            </button>
        </section>
    }
    .into_any()
}

fn title_field(draft: Draft) -> AnyView {
    view! { <label>{t("recipe.label.title")} <input value=move || draft.title.get() on:input=move |ev| draft.title.set(event_target_value(&ev)) required/></label> }.into_any()
}

fn servings_field(draft: Draft) -> AnyView {
    view! { <label>{t("recipe.label.servings")} <input type="number" min="1" step="1" value=move || draft.servings.get() on:input=move |ev| draft.servings.set(event_target_value(&ev)) required/></label> }.into_any()
}

/// The categories of the recipe form, one checkbox each, checked for the categories of the recipe.
#[component]
pub(crate) fn CategoryBoxes(draft: Draft, categories: Vec<CategoryRow>) -> impl IntoView {
    view! {
        <details class="recipe-metadata">
            <summary>{t("recipe.section.metadata")}</summary>
            <fieldset>
                <legend>{t("recipe.categories.heading")}</legend>
                {categories.into_iter().map(|category| category_box(draft, category)).collect_view()}
            </fieldset>
        </details>
    }
}

fn category_box(draft: Draft, category: CategoryRow) -> impl IntoView {
    let id = category.id;
    view! {
        <label>
            <input type="checkbox" value=id checked=move || draft.categories.get().contains(&id) on:change=move |ev| draft.set_category(id, event_target_checked(&ev))/>
            {category.name}
        </label>
    }
}

fn line_row(draft: Draft, line: LineSignals, unresolved: Vec<String>) -> AnyView {
    view! {
        <section class="card recipe-row" class:unresolved=move || unresolved.contains(&line.ingredient.get())>
            <label>{t("recipe.label.amount")} <input type="number" min="0" step="any" value=move || line.amount.get() on:input=move |ev| line.amount.set(event_target_value(&ev))/></label>
            <label>{t("recipe.label.unit")}
                <select on:change=move |ev| line.unit.set(event_target_value(&ev))>
                    <option value="" selected=move || line.unit.get().is_empty()>{t("recipe.unit.none")}</option>
                    {Unit::ALL.into_iter().map(|unit| view! {
                        <option value=unit.symbol() selected=move || line.unit.get() == unit.symbol()>{unit_text(messages(), unit.symbol())}</option>
                    }).collect_view()}
                </select>
            </label>
            <label>{t("recipe.label.ingredient")} <input list="ingredient-names" value=move || line.ingredient.get() on:input=move |ev| line.ingredient.set(event_target_value(&ev))/></label>
            <label>{t("recipe.label.note")} <input value=move || line.note.get() on:input=move |ev| line.note.set(event_target_value(&ev))/></label>
            <button type="button" class="secondary row-remove" aria-label=t("recipe.button.remove_ingredient") on:click=move |_| draft.remove_line(line.key)>
                {t("recipe.button.remove")}
            </button>
        </section>
    }
    .into_any()
}

fn step_row(draft: Draft, step: StepSignals) -> AnyView {
    view! {
        <section class="card recipe-row step-row">
            <label>{t("recipe.label.step")} <textarea on:input=move |ev| step.text.set(event_target_value(&ev))>{move || step.text.get()}</textarea></label>
            <label>{t("recipe.label.minutes")} <input type="number" min="1" step="1" value=move || step.minutes.get() on:input=move |ev| step.minutes.set(event_target_value(&ev))/></label>
            {move || (!draft.lines.get().is_empty()).then(move || view! {
                <fieldset>
                    <legend>{t("recipe.step.ingredients")}</legend>
                    <For each=move || draft.lines.get() key=|row| row.key children=move |line| step_line(step, line)/>
                </fieldset>
            })}
            <button type="button" class="secondary row-remove" aria-label=t("recipe.button.remove_step") on:click=move |_| draft.remove_step(step.key)>
                {t("recipe.button.remove")}
            </button>
        </section>
    }
    .into_any()
}

fn step_line(step: StepSignals, line: LineSignals) -> AnyView {
    view! {
        <label>
            <input type="checkbox" checked=move || step.lines.get().contains(&line.key) on:change=move |ev| step.set_line(line.key, event_target_checked(&ev))/>
            {move || line.text()}
        </label>
    }
    .into_any()
}
