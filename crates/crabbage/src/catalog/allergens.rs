//! The allergens of catalogue ingredients: their names and the admin endpoint that sets them.

#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces each server function body with a stub that does not await"
    )
)]

use std::collections::HashMap;

use crabbage_domain::allergens::Allergen;
use leptos::prelude::ServerFnError;
use leptos::server;

/// The locale key of an allergen's name.
pub(crate) const fn allergen_key(allergen: Allergen) -> &'static str {
    match allergen {
        Allergen::Gluten => "allergen.gluten",
        Allergen::Crustaceans => "allergen.crustaceans",
        Allergen::Eggs => "allergen.eggs",
        Allergen::Fish => "allergen.fish",
        Allergen::Peanuts => "allergen.peanuts",
        Allergen::Soy => "allergen.soy",
        Allergen::Milk => "allergen.milk",
        Allergen::TreeNuts => "allergen.tree_nuts",
        Allergen::Celery => "allergen.celery",
        Allergen::Mustard => "allergen.mustard",
        Allergen::Sesame => "allergen.sesame",
        Allergen::Sulphites => "allergen.sulphites",
        Allergen::Lupin => "allergen.lupin",
        Allergen::Molluscs => "allergen.molluscs",
        Allergen::Meat => "allergen.meat",
    }
}

/// Replaces the allergens of the ingredient `id` with the checked boxes, sent as
/// `allergens[<code>]`. No boxes clears them; unknown codes are ignored.
#[server(prefix = "/api", endpoint = "set_ingredient_allergens")]
pub(crate) async fn set_ingredient_allergens(
    id: i64,
    allergens: Option<HashMap<String, String>>,
) -> Result<(), ServerFnError> {
    let pool = crate::access::require_admin().await?;
    let checked = Allergen::from_codes(allergens.iter().flatten().map(|(code, _)| code.as_str()));
    let found = crabbage_store::catalog::allergens::set_allergens(&pool, id, &checked)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    crate::access::ensure_found(found)
}
