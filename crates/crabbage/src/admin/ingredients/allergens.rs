//! The allergen boxes of one ingredient, shared by its page and the review queue.

use crabbage_domain::allergens::Allergen;
use leptos::form::ActionForm;
use leptos::prelude::*;

use crate::catalog::allergens::{SetIngredientAllergens, allergen_key};
use crate::i18n::t;

/// A form with one box per allergen, the ingredient's own checked, and a save button.
pub(crate) fn allergen_form(
    id: i64,
    allergens: &[String],
    action: ServerAction<SetIngredientAllergens>,
) -> AnyView {
    let boxes = Allergen::ALL
        .into_iter()
        .map(|allergen| {
            let code = allergen.code();
            let checked = allergens.iter().any(|own| own == code);
            view! {
                <label>
                    <input type="checkbox" name=format!("allergens[{code}]") checked=checked/>
                    {t(allergen_key(allergen))}
                </label>
            }
        })
        .collect_view();
    view! {
        <ActionForm action=action attr:class="admin-allergens">
            <input type="hidden" name="id" value=id/>
            <fieldset>
                <legend>{t("admin.label.allergens")}</legend>
                {boxes}
            </fieldset>
            <button>{t("admin.button.save")}</button>
        </ActionForm>
    }
    .into_any()
}
