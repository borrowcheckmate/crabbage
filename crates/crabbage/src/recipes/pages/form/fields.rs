//! The fields of the recipe form: the title, the servings, the ingredient rows and the step rows,
//! each row with a button to remove it.

use crabbage_domain::quantity::Unit;
use leptos::prelude::event_target_value;
use leptos::prelude::*;

use super::draft::{Draft, LineSignals, StepSignals};
use crate::i18n::{messages, t};
use crate::recipes::types::{LineDraft, StepDraft};
use crate::recipes::units::unit_text;

/// The fields of the recipe form. The names fill the suggestions of the ingredient fields.
#[component]
pub(crate) fn DraftFields(draft: Draft, names: Vec<String>) -> impl IntoView {
    let suggestions = names
        .into_iter()
        .map(|name| view! { <option value=name/> })
        .collect_view();
    view! {
        <label>{t("recipe.label.title")} <input value=move || draft.title.get() on:input=move |ev| draft.title.set(event_target_value(&ev)) required/></label>
        <label>{t("recipe.label.servings")} <input type="number" min="1" step="1" value=move || draft.servings.get() on:input=move |ev| draft.servings.set(event_target_value(&ev)) required/></label>
        <h2>{t("recipe.lines.heading")}</h2>
        <div class="rows">
            <For each=move || draft.lines.get() key=|row| row.key children=move |row| line_row(draft, row)/>
        </div>
        <button type="button" class="secondary" on:click=move |_| draft.add_line(LineDraft::default())>
            {t("recipe.button.add_line")}
        </button>
        <h2>{t("recipe.steps.heading")}</h2>
        <div class="rows">
            <For each=move || draft.steps.get() key=|row| row.key children=move |row| step_row(draft, row)/>
        </div>
        <button type="button" class="secondary" on:click=move |_| draft.add_step(StepDraft::default())>
            {t("recipe.button.add_step")}
        </button>
        <datalist id="ingredient-names">{suggestions}</datalist>
    }
}

fn line_row(draft: Draft, line: LineSignals) -> impl IntoView {
    view! {
        <section class="card">
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
            <button type="button" class="secondary" on:click=move |_| draft.remove_line(line.key)>
                {t("recipe.button.remove")}
            </button>
        </section>
    }
}

fn step_row(draft: Draft, step: StepSignals) -> impl IntoView {
    view! {
        <section class="card">
            <label>{t("recipe.label.step")} <textarea on:input=move |ev| step.text.set(event_target_value(&ev))>{move || step.text.get()}</textarea></label>
            <label>{t("recipe.label.minutes")} <input type="number" min="1" step="1" value=move || step.minutes.get() on:input=move |ev| step.minutes.set(event_target_value(&ev))/></label>
            <button type="button" class="secondary" on:click=move |_| draft.remove_step(step.key)>
                {t("recipe.button.remove")}
            </button>
        </section>
    }
}
