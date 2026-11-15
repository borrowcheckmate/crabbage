//! The names of the dietary warnings, from the locale, the line naming a recipe's warnings and the
//! line naming what in it the household avoids.

use crabbage_domain::allergens::Allergen;
use crabbage_domain::warnings::Warning;
use leptos::prelude::*;

use crate::catalog::allergens::allergen_key;
use crate::i18n::t;

/// The locale key of a dietary warning's name.
pub(crate) const fn warning_key(warning: Warning) -> &'static str {
    match warning {
        Warning::RawEgg => "warning.raw_egg",
        Warning::RawFish => "warning.raw_fish",
        Warning::RareMeat => "warning.rare_meat",
        Warning::UnpasteurisedCheese => "warning.unpasteurised_cheese",
        Warning::Liver => "warning.liver",
        Warning::Alcohol => "warning.alcohol",
    }
}

/// "Not advised in pregnancy: …" with the names of the warning `codes`, or nothing without any.
pub(crate) fn warning_line(codes: &[String]) -> Option<AnyView> {
    let names: Vec<&str> = codes
        .iter()
        .filter_map(|code| Warning::from_code(code))
        .map(|warning| t(warning_key(warning)))
        .collect();
    (!names.is_empty()).then(|| {
        view! {
            <p class="recipe-warnings"><strong>{t("recipe.warnings")}</strong>{format!(" {}", names.join(", "))}</p>
        }
        .into_any()
    })
}

/// "Your household avoids: …" with the names of the allergen and warning `codes`, or nothing
/// without any.
pub(crate) fn avoided_line(codes: &[String]) -> Option<AnyView> {
    let names: Vec<&str> = codes
        .iter()
        .filter_map(|code| {
            Allergen::from_code(code)
                .map(allergen_key)
                .or_else(|| Warning::from_code(code).map(warning_key))
        })
        .map(t)
        .collect();
    (!names.is_empty()).then(|| {
        view! {
            <p class="recipe-avoided" role="note"><strong>{t("recipe.avoided")}</strong>{format!(" {}", names.join(", "))}</p>
        }
        .into_any()
    })
}
