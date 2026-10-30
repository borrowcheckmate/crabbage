//! The recipe id in the address of a recipe page.

use leptos::prelude::With;
use leptos_router::hooks::use_params_map;

/// The `id` of the recipe route, or `None` when it is not a number.
pub(crate) fn recipe_id() -> impl Fn() -> Option<i64> + Copy {
    let params = use_params_map();
    move || {
        params
            .with(|map| map.get("id"))
            .and_then(|id| id.parse().ok())
    }
}
