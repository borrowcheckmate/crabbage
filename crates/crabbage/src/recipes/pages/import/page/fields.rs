use leptos::prelude::event_target_value;
use leptos::prelude::*;

use super::super::super::super::form::{Draft, LineSignals, StepSignals};
use super::{count_label, format_amount};
use crate::i18n::t;
use crate::recipes::types::{LineDraft, StepDraft};

pub(super) fn import_fields(draft: Draft, names: Vec<String>, unresolved: Vec<String>) -> AnyView {
    let options = names
        .into_iter()
        .map(|name| view! { <option value=name/> })
        .collect_view();
    view! {
        <div class="import-metaform">
            {title_field(draft)}
            {servings_field(draft)}
        </div>
        <section class="import-ingredients">
            <header><h2>{t("recipe.lines.heading")}</h2><span>{move || count_label("recipe.ingredients.one", "recipe.ingredients.many", draft.lines.get().len())}</span></header>
            <For each=move || draft.lines.get() key=|line| line.key children=move |line| import_line(draft, line, unresolved.clone())/>
            <button type="button" class="import-add" on:click=move |_| draft.add_line(LineDraft::default())>
                <svg aria-hidden="true" viewBox="0 0 24 24"><path d="M12 5v14M5 12h14"/></svg>{t("recipe.button.add_line")}
            </button>
            <datalist id="import-ingredient-names">{options}</datalist>
        </section>
        <details class="import-method">
            <summary>{t("recipe.steps.heading")} <span>{move || count_label("import.steps.one", "import.steps.many", draft.steps.get().len())}</span></summary>
            <For each=move || draft.steps.get() key=|step| step.key children=move |step| import_step(draft, step)/>
            <button type="button" class="import-add" on:click=move |_| draft.add_step(StepDraft::default())>
                <svg aria-hidden="true" viewBox="0 0 24 24"><path d="M12 5v14M5 12h14"/></svg>{t("recipe.button.add_step")}
            </button>
        </details>
    }
    .into_any()
}

fn title_field(draft: Draft) -> AnyView {
    view! { <label>{t("import.title_label")}<input value=move || draft.title.get() on:input=move |ev| draft.title.set(event_target_value(&ev)) required/></label> }.into_any()
}

fn servings_field(draft: Draft) -> AnyView {
    view! { <label>{t("recipe.label.servings")}<input type="number" min="1" step="1" value=move || draft.servings.get() on:input=move |ev| draft.servings.set(event_target_value(&ev)) required/></label> }.into_any()
}

fn import_line(draft: Draft, line: LineSignals, unresolved: Vec<String>) -> AnyView {
    let unresolved_class = unresolved.clone();
    view! {
        <details class="import-line" class:unresolved=move || unresolved_class.contains(&line.ingredient.get())>
            <summary>
                <span class="import-amount">{move || format_amount(&line.amount.get(), &line.unit.get())}</span>
                <span class="import-line-name"><strong>{move || line.ingredient.get()}</strong>
                    {move || unresolved.contains(&line.ingredient.get()).then(|| view! { <span class="import-new">{t("import.new_ingredient")}</span> })}
                </span>
                <svg class="import-pencil" aria-hidden="true" viewBox="0 0 24 24"><path d="m15 5 4 4M4 20l4-.8L19 8a2.8 2.8 0 0 0-4-4L4 15z"/></svg>
            </summary>
            <div class="import-line-editor">
                {amount_field(line)}
                {unit_field(line)}
                {ingredient_field(line)}
                {note_field(line)}
                <button type="button" class="secondary" aria-label=t("recipe.button.remove_ingredient") on:click=move |_| draft.remove_line(line.key)>{t("recipe.button.remove")}</button>
            </div>
        </details>
    }
    .into_any()
}

fn amount_field(line: LineSignals) -> AnyView {
    view! { <label>{t("recipe.label.amount")}<input type="number" min="0" step="any" value=move || line.amount.get() on:input=move |ev| line.amount.set(event_target_value(&ev))/></label> }.into_any()
}

fn unit_field(line: LineSignals) -> AnyView {
    view! { <label>{t("recipe.label.unit")}<input value=move || line.unit.get() on:input=move |ev| line.unit.set(event_target_value(&ev))/></label> }.into_any()
}

fn ingredient_field(line: LineSignals) -> AnyView {
    view! { <label>{t("recipe.label.ingredient")}<input list="import-ingredient-names" value=move || line.ingredient.get() on:input=move |ev| line.ingredient.set(event_target_value(&ev))/></label> }.into_any()
}

fn note_field(line: LineSignals) -> AnyView {
    view! { <label>{t("recipe.label.note")}<input value=move || line.note.get() on:input=move |ev| line.note.set(event_target_value(&ev))/></label> }.into_any()
}

fn import_step(draft: Draft, step: StepSignals) -> AnyView {
    view! {
        <section class="import-step-row">
            <label class="import-step">{t("recipe.label.step")}<textarea on:input=move |ev| step.text.set(event_target_value(&ev))>{move || step.text.get()}</textarea></label>
            <label class="import-minutes">{t("recipe.label.minutes")}<input type="number" min="1" step="1" value=move || step.minutes.get() on:input=move |ev| step.minutes.set(event_target_value(&ev))/></label>
            <button type="button" class="secondary" aria-label=t("recipe.button.remove_step") on:click=move |_| draft.remove_step(step.key)>{t("recipe.button.remove")}</button>
        </section>
    }
    .into_any()
}
