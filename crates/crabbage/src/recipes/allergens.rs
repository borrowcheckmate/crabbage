//! The line naming what a recipe contains, shown on the recipe page and in cook mode.

use crabbage_domain::allergens::Allergen;
use leptos::prelude::*;

use crate::catalog::allergens::allergen_key;
use crate::i18n::t;

/// "Contains: …" with the names of the allergen `codes`, or nothing when there are none.
pub(crate) fn allergen_line(codes: &[String]) -> Option<AnyView> {
    let names: Vec<&str> = codes
        .iter()
        .filter_map(|code| Allergen::from_code(code))
        .map(|allergen| t(allergen_key(allergen)))
        .collect();
    (!names.is_empty()).then(|| {
        view! {
            <p class="recipe-allergens"><strong>{t("recipe.allergens")}</strong>{format!(" {}", names.join(", "))}</p>
        }
        .into_any()
    })
}
