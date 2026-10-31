//! The recipe list: a row of filters, then a card per recipe that passes them and the button for a
//! new one.

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::catalog::CategoryRow;
use crate::i18n::t;
use crate::notice::{error_notice, message};
use crate::recipes::categories::list_recipe_categories;
use crate::recipes::list::list_recipes;
use crate::recipes::rating_text::rating_text;
use crate::recipes::types::RecipeSummaryRow;

/// The recipe list. Every signed-in user sees every recipe, narrowed by the `category` and
/// `favourites` values in the address.
#[component]
pub(crate) fn RecipesPage() -> impl IntoView {
    let query = use_query_map();
    view! {
        <h1>{t("recipes.heading")}</h1>
        <a class="button" href="/recipes/new">{t("recipes.new")}</a>
        <Suspense fallback=|| ()>
            {move || {
                let category = query.with(|map| map.get("category")).and_then(|id| id.parse().ok());
                let favourites = query.with(|map| map.get("favourites")).as_deref() == Some("1");
                Suspend::new(async move { recipe_list(category, favourites).await })
            }}
        </Suspense>
    }
}

async fn recipe_list(category: Option<i64>, favourites: bool) -> AnyView {
    match load(category, favourites).await {
        Ok((categories, rows)) => {
            let empty = if favourites {
                t("recipes.empty_favourites")
            } else if category.is_some() {
                t("recipes.empty_category")
            } else {
                t("recipes.empty")
            };
            view! {
                {filters(categories, category, favourites)}
                {if rows.is_empty() {
                    view! { <p>{empty}</p> }.into_any()
                } else {
                    rows.into_iter().map(recipe_card).collect_view().into_any()
                }}
            }
            .into_any()
        }
        Err(error) => error_notice(&message(&error)),
    }
}

async fn load(
    category: Option<i64>,
    favourites: bool,
) -> Result<(Vec<CategoryRow>, Vec<RecipeSummaryRow>), ServerFnError> {
    Ok((
        list_recipe_categories().await?,
        list_recipes(category, favourites).await?,
    ))
}

/// The address of the list with the given filters.
fn list_href(category: Option<i64>, favourites: bool) -> String {
    let mut parts = Vec::new();
    if let Some(id) = category {
        parts.push(format!("category={id}"));
    }
    if favourites {
        parts.push("favourites=1".to_owned());
    }
    if parts.is_empty() {
        "/recipes".to_owned()
    } else {
        format!("/recipes?{}", parts.join("&"))
    }
}

/// The filter links: All, the favourites toggle, then one per category. The active ones are marked
/// for assistive tech. All clears both filters; the others keep the filter they do not change.
fn filters(categories: Vec<CategoryRow>, active: Option<i64>, favourites: bool) -> impl IntoView {
    let unfiltered = active.is_none() && !favourites;
    let toggle_href = list_href(active, !favourites);
    view! {
        <nav class="row filters" aria-label=t("recipes.categories.label")>
            <a href="/recipes" aria-current=unfiltered.then_some("page")>{t("recipes.category.all")}</a>
            <a href=toggle_href aria-current=favourites.then_some("page")>{t("recipes.filter.favourites")}</a>
            {categories
                .into_iter()
                .map(|category| {
                    let href = list_href(Some(category.id), favourites);
                    let current = (active == Some(category.id)).then_some("page");
                    view! { <a href=href aria-current=current>{category.name}</a> }
                })
                .collect_view()}
        </nav>
    }
}

fn recipe_card(row: RecipeSummaryRow) -> impl IntoView {
    let href = format!("/recipes/{}", row.id);
    let servings = format!("{}: {}", t("recipe.servings"), row.servings);
    let owner = format!("{}: {}", t("recipe.added_by"), row.owner_name);
    let rating = rating_text(row.average, row.rating_count);
    view! {
        <section class="card">
            <h2><a href=href>{row.title}</a> {row.favourite.then(|| view! { <span class="badge">{t("recipe.favourite.badge")}</span> })}</h2>
            <p>{rating}</p>
            <p>{servings}</p>
            <p>{owner}</p>
        </section>
    }
}
