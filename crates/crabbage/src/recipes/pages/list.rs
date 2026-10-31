//! The recipe list: a search box, a row of filters, then a card per recipe that passes them and the
//! button for a new one.

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::catalog::CategoryRow;
use crate::i18n::{t, t_with};
use crate::notice::{error_notice, message};
use crate::recipes::categories::list_recipe_categories;
use crate::recipes::list::list_recipes;
use crate::recipes::rating_text::rating_text;
use crate::recipes::types::RecipeSummaryRow;

/// The recipe list. Every signed-in user sees every recipe, narrowed by the `category`,
/// `favourites` and `q` values in the address.
#[component]
pub(crate) fn RecipesPage() -> impl IntoView {
    let query = use_query_map();
    view! {
        <h1>{t("recipes.heading")}</h1>
        <a class="button" href="/recipes/new">{t("recipes.new")}</a>
        <Suspense fallback=|| ()>
            {move || {
                let filters = query.with(|map| Filters {
                    category: map.get("category").and_then(|id| id.parse().ok()),
                    favourites: map.get("favourites").as_deref() == Some("1"),
                    search: map.get("q").unwrap_or_default().trim().to_owned(),
                });
                Suspend::new(async move { recipe_list(filters).await })
            }}
        </Suspense>
    }
}

/// The filters the address names: a category, the favourites alone and a search text.
#[derive(Clone, Default)]
struct Filters {
    category: Option<i64>,
    favourites: bool,
    search: String,
}

impl Filters {
    /// The address of the list with these filters.
    fn href(&self) -> String {
        let mut parts = Vec::new();
        if let Some(id) = self.category {
            parts.push(format!("category={id}"));
        }
        if self.favourites {
            parts.push("favourites=1".to_owned());
        }
        if !self.search.is_empty() {
            parts.push(format!("q={}", encode(&self.search)));
        }
        if parts.is_empty() {
            "/recipes".to_owned()
        } else {
            format!("/recipes?{}", parts.join("&"))
        }
    }
}

async fn recipe_list(filters: Filters) -> AnyView {
    match load(&filters).await {
        Ok((categories, rows)) => {
            let empty = empty_text(&filters);
            view! {
                {search_form(&filters)}
                {filter_links(categories, &filters)}
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
    filters: &Filters,
) -> Result<(Vec<CategoryRow>, Vec<RecipeSummaryRow>), ServerFnError> {
    Ok((
        list_recipe_categories().await?,
        list_recipes(
            filters.category,
            filters.favourites,
            Some(filters.search.clone()),
        )
        .await?,
    ))
}

/// The message for a list without rows: it names the search, otherwise it explains the filter.
fn empty_text(filters: &Filters) -> String {
    if !filters.search.is_empty() {
        t_with("recipes.empty_search", &[("q", &filters.search)])
    } else if filters.favourites {
        t("recipes.empty_favourites").to_owned()
    } else if filters.category.is_some() {
        t("recipes.empty_category").to_owned()
    } else {
        t("recipes.empty").to_owned()
    }
}

/// The search form: a search box for `q`, and hidden fields that keep the category and favourites
/// filters when the search is submitted.
fn search_form(filters: &Filters) -> impl IntoView {
    let category = filters
        .category
        .map(|id| view! { <input type="hidden" name="category" value=id/> });
    let favourites = filters
        .favourites
        .then(|| view! { <input type="hidden" name="favourites" value="1"/> });
    view! {
        <form method="get" action="/recipes" role="search">
            {category}
            {favourites}
            <label>{t("recipes.search.label")} <input type="search" name="q" value=filters.search.clone()/></label>
            <button>{t("recipes.search.button")}</button>
        </form>
    }
}

/// The filter links: All, the favourites toggle, then one per category. The active ones are marked
/// for assistive tech. Every link keeps the search text; All clears the other two filters and the
/// others keep the filter they do not change.
fn filter_links(categories: Vec<CategoryRow>, filters: &Filters) -> impl IntoView {
    let unfiltered = filters.category.is_none() && !filters.favourites;
    let all = Filters {
        category: None,
        favourites: false,
        ..filters.clone()
    }
    .href();
    let toggle = Filters {
        favourites: !filters.favourites,
        ..filters.clone()
    }
    .href();
    view! {
        <nav class="row filters" aria-label=t("recipes.categories.label")>
            <a href=all aria-current=unfiltered.then_some("page")>{t("recipes.category.all")}</a>
            <a href=toggle aria-current=filters.favourites.then_some("page")>{t("recipes.filter.favourites")}</a>
            {categories
                .into_iter()
                .map(|category| {
                    let current = (filters.category == Some(category.id)).then_some("page");
                    let href = Filters {
                        category: Some(category.id),
                        ..filters.clone()
                    }
                    .href();
                    view! { <a href=href aria-current=current>{category.name}</a> }
                })
                .collect_view()}
        </nav>
    }
}

/// The text of a query value: letters, digits and `-_.~` as they are, every other byte as `%XX`.
fn encode(text: &str) -> String {
    text.bytes()
        .map(|byte| match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                char::from(byte).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
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
