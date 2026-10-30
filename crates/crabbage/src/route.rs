//! The id in the address of a page such as `/recipes/:id` or `/admin/ingredients/:id`.

use leptos::prelude::With;
use leptos_router::hooks::use_params_map;

/// The `id` of the current route, or `None` when it is not a number.
pub(crate) fn route_id() -> impl Fn() -> Option<i64> + Copy {
    let params = use_params_map();
    move || {
        params
            .with(|map| map.get("id"))
            .and_then(|id| id.parse().ok())
    }
}
