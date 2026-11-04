//! The document title: the page's name before the app's, so tabs and history entries tell pages
//! apart. Pages that load a name of their own, such as a recipe, replace the route's name with it.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_location;

use crate::i18n::t;

/// The title for the current route, and the formatter that adds the app's name to every title.
#[component]
pub(crate) fn RouteTitle() -> impl IntoView {
    let location = use_location();
    view! {
        <Title formatter=|page: String| full_title(&page)/>
        <Title text=move || location.pathname.with(|path| route_title_key(path).map(t).unwrap_or_default().to_owned())/>
    }
}

/// `"{page} · {app}"`, or the app's name alone when the page has no name or is named like the app.
pub(crate) fn full_title(page: &str) -> String {
    let app = t("app.title");
    if page.trim().is_empty() || page == app {
        app.to_owned()
    } else {
        format!("{page} · {app}")
    }
}

/// The translation key that names the page at `path`, or `None` for addresses with no page.
pub(crate) fn route_title_key(path: &str) -> Option<&'static str> {
    let segments: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();
    match segments.as_slice() {
        [] => Some("home.heading"),
        ["shopping"] => Some("shopping.heading"),
        ["plan"] => Some("plan.heading"),
        ["recipes", "new"] => Some("recipe.new_heading"),
        ["recipes", "import"] => Some("import.heading"),
        ["recipes"] | ["recipes", _] => Some("recipes.heading"),
        ["recipes", _, "cook"] => Some("cook.heading"),
        ["recipes", _, "edit"] => Some("recipe.edit_heading"),
        ["admin", "ingredients", ..] => Some("admin.ingredients.heading"),
        ["admin", "sections"] => Some("admin.sections.heading"),
        ["admin", "categories"] => Some("admin.categories.heading"),
        ["admin", "review"] => Some("admin.review.heading"),
        ["admin", "mealie"] => Some("admin.mealie.heading"),
        ["admin", "settings"] => Some("admin.settings.heading"),
        ["admin", "households"] => Some("admin.households.heading"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{full_title, route_title_key};

    #[test]
    fn every_route_is_named() {
        for (path, key) in [
            ("/", "home.heading"),
            ("/shopping", "shopping.heading"),
            ("/plan", "plan.heading"),
            ("/recipes", "recipes.heading"),
            ("/recipes/", "recipes.heading"),
            ("/recipes/new", "recipe.new_heading"),
            ("/recipes/import", "import.heading"),
            ("/recipes/12", "recipes.heading"),
            ("/recipes/12/cook", "cook.heading"),
            ("/recipes/12/edit", "recipe.edit_heading"),
            ("/admin/ingredients", "admin.ingredients.heading"),
            ("/admin/ingredients/4", "admin.ingredients.heading"),
            ("/admin/sections", "admin.sections.heading"),
            ("/admin/categories", "admin.categories.heading"),
            ("/admin/review", "admin.review.heading"),
            ("/admin/mealie", "admin.mealie.heading"),
            ("/admin/settings", "admin.settings.heading"),
            ("/admin/households", "admin.households.heading"),
        ] {
            assert_eq!(route_title_key(path), Some(key), "{path}");
        }
    }

    #[test]
    fn unknown_addresses_have_no_name() {
        for path in [
            "/nowhere",
            "/admin",
            "/recipes/12/cook/more",
            "/shopping/items",
        ] {
            assert_eq!(route_title_key(path), None, "{path}");
        }
    }

    #[test]
    fn the_page_comes_before_the_app() {
        assert_eq!(full_title("Shopping list"), "Shopping list · crabbage");
        assert_eq!(full_title(""), "crabbage");
        assert_eq!(full_title("  "), "crabbage");
        assert_eq!(full_title("crabbage"), "crabbage");
    }
}
