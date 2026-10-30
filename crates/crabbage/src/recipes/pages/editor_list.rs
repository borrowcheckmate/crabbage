//! The editors of a recipe: who may change it, with a button to remove each one, and a choice of
//! the users who are not editors yet.

use leptos::form::ActionForm;
use leptos::prelude::*;

use crate::i18n::t;
use crate::notice::failure;
use crate::recipes::editors::{AddRecipeEditor, RemoveRecipeEditor};
use crate::recipes::types::PersonRow;

/// The editors section. Adding and removing update the list in place, so the form on the page keeps
/// what was typed.
#[component]
pub(crate) fn EditorsSection(
    recipe_id: i64,
    editors: Vec<PersonRow>,
    users: Vec<PersonRow>,
) -> impl IntoView {
    let current = RwSignal::new(editors);
    let add = ServerAction::<AddRecipeEditor>::new();
    let remove = ServerAction::<RemoveRecipeEditor>::new();
    Effect::new(move |_| {
        add.value().with(|value| {
            if let Some(Ok(list)) = value {
                current.set(list.clone());
            }
        });
    });
    Effect::new(move |_| {
        remove.value().with(|value| {
            if let Some(Ok(list)) = value {
                current.set(list.clone());
            }
        });
    });
    view! {
        <section class="card">
        <h2>{t("recipe.editors.heading")}</h2>
        <ul>
            {move || current.get().into_iter().map(|person| {
                view! {
                    <li>
                        <ActionForm action=remove>
                            <input type="hidden" name="recipe_id" value=recipe_id/>
                            <input type="hidden" name="user_id" value=person.id/>
                            <span>{person.name}</span>
                            <button class="secondary">{t("recipe.button.remove")}</button>
                        </ActionForm>
                    </li>
                }
            }).collect_view()}
        </ul>
        {move || {
            let taken: Vec<i64> = current.get().iter().map(|person| person.id).collect();
            let free: Vec<PersonRow> = users
                .iter()
                .filter(|person| !taken.contains(&person.id))
                .cloned()
                .collect();
            (!free.is_empty()).then(|| view! {
                <ActionForm action=add>
                    <input type="hidden" name="recipe_id" value=recipe_id/>
                    <label>{t("recipe.editors.pick")}
                        <select name="user_id">
                            {free.iter().map(|person| view! {
                                <option value=person.id.to_string()>{person.name.clone()}</option>
                            }).collect_view()}
                        </select>
                    </label>
                    <button>{t("recipe.button.add_editor")}</button>
                </ActionForm>
            })
        }}
        {move || add.value().with(|value| failure(value.as_ref()))}
        {move || remove.value().with(|value| failure(value.as_ref()))}
        </section>
    }
}
