//! The recipe list: a row of category filters, then a card per recipe in the chosen category and the
//! button for a new one.

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::catalog::CategoryRow;
use crate::i18n::t;
use crate::notice::{error_notice, message};
use crate::recipes::categories::list_recipe_categories;
use crate::recipes::list::list_recipes;
use crate::recipes::types::RecipeSummaryRow;

/// The recipe list. Every signed-in user sees every recipe, or the ones in the category that the
/// address names with `category`.
#[component]
pub(crate) fn RecipesPage() -> impl IntoView {
    let query = use_query_map();
    view! {
        <h1>{t("recipes.heading")}</h1>
        <a class="button" href="/recipes/new">{t("recipes.new")}</a>
        <Suspense fallback=|| ()>
            {move || {
                let category = query.with(|map| map.get("category")).and_then(|id| id.parse().ok());
                Suspend::new(async move { recipe_list(category).await })
            }}
        </Suspense>
    }
}

async fn recipe_list(category: Option<i64>) -> AnyView {
    match load(category).await {
        Ok((categories, rows)) => {
            let empty = if category.is_some() {
                t("recipes.empty_category")
            } else {
                t("recipes.empty")
            };
            view! {
                {filters(categories, category)}
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
) -> Result<(Vec<CategoryRow>, Vec<RecipeSummaryRow>), ServerFnError> {
    Ok((
        list_recipe_categories().await?,
        list_recipes(category).await?,
    ))
}

/// The filter links: All, then one per category. The active one is marked for assistive tech.
fn filters(categories: Vec<CategoryRow>, active: Option<i64>) -> impl IntoView {
    view! {
        <nav class="row filters" aria-label=t("recipes.categories.label")>
            <a href="/recipes" aria-current=active.is_none().then_some("page")>{t("recipes.category.all")}</a>
            {categories
                .into_iter()
                .map(|category| {
                    let href = format!("/recipes?category={}", category.id);
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
    view! {
        <section class="card">
            <h2><a href=href>{row.title}</a></h2>
            <p>{servings}</p>
            <p>{owner}</p>
        </section>
    }
}
