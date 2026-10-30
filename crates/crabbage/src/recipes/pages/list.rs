//! The recipe list: a card per recipe and the button for a new one.

use leptos::prelude::*;

use crate::i18n::t;
use crate::notice::{error_notice, message};
use crate::recipes::list::list_recipes;
use crate::recipes::types::RecipeSummaryRow;

/// The recipe list. Every signed-in user sees every recipe.
#[component]
pub(crate) fn RecipesPage() -> impl IntoView {
    view! {
        <h1>{t("recipes.heading")}</h1>
        <a class="button" href="/recipes/new">{t("recipes.new")}</a>
        <Suspense fallback=|| ()>
            {move || Suspend::new(async move { recipe_list().await })}
        </Suspense>
    }
}

async fn recipe_list() -> AnyView {
    match list_recipes().await {
        Ok(rows) if rows.is_empty() => view! { <p>{t("recipes.empty")}</p> }.into_any(),
        Ok(rows) => rows.into_iter().map(recipe_card).collect_view().into_any(),
        Err(error) => error_notice(&message(&error)),
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
