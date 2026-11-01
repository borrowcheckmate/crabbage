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

type RecipeListResult = Result<(Vec<CategoryRow>, Vec<RecipeSummaryRow>), ServerFnError>;
type RecipeListData = (Filters, RecipeListResult);

/// The recipe list. Every signed-in user sees every recipe, narrowed by the `category`,
/// `favourites` and `q` values in the address.
#[component]
pub(crate) fn RecipesPage() -> impl IntoView {
    let query = use_query_map();
    let data = Resource::new(
        move || {
            query.with(|map| Filters {
                category: map.get("category").and_then(|id| id.parse().ok()),
                favourites: map.get("favourites").as_deref() == Some("1"),
                search: map.get("q").unwrap_or_default().trim().to_owned(),
            })
        },
        load,
    );
    view! {
        <header class="recipe-list-heading">
            <h1>{t("recipes.heading")}</h1>
            <div class="row"><a class="button" href="/recipes/new">{t("recipes.new")}</a>
            <a class="button secondary" href="/recipes/import">{t("recipes.import")}</a></div>
        </header>
        <Suspense fallback=|| ()>
            {move || {
                Suspend::new(async move { recipe_list(data.await) })
            }}
        </Suspense>
    }
}

/// The filters the address names: a category, the favourites alone and a search text.
#[derive(Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
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

fn recipe_list((filters, result): RecipeListData) -> AnyView {
    match result {
        Ok((categories, rows)) => {
            let empty = empty_text(&filters);
            view! {
                <div class="recipe-list-content">
                {search_form(&filters)}
                {filter_links(categories, &filters)}
                {if rows.is_empty() {
                    view! { <p class="recipe-empty">{empty}</p> }.into_any()
                } else {
                    view! { <div class="recipe-grid">{rows.into_iter().enumerate().map(|(index, row)| recipe_card(row, &filters, index == 0)).collect_view()}</div> }.into_any()
                }}
                </div>
            }
            .into_any()
        }
        Err(error) => error_notice(&message(&error)),
    }
}

async fn load(filters: Filters) -> RecipeListData {
    let result = load_rows(&filters).await;
    (filters, result)
}

async fn load_rows(filters: &Filters) -> RecipeListResult {
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
        <form class="recipe-search-form" method="get" action="/recipes" role="search">
            {category}
            {favourites}
            <label class="recipe-search-label"><span class="recipe-search-icon" aria-hidden="true"><svg viewBox="0 0 24 24"><circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/></svg></span><input type="search" name="q" aria-label=t("recipes.search.label") placeholder=t("recipes.search.placeholder") value=filters.search.clone()/></label>
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

fn encode_hex(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len().saturating_mul(2));
    for byte in text.bytes() {
        for digit in [byte >> 4, byte & 0x0f] {
            if let Some(character) = char::from_digit(u32::from(digit), 16) {
                encoded.push(character);
            }
        }
    }
    encoded
}

fn recipe_card(row: RecipeSummaryRow, filters: &Filters, first: bool) -> impl IntoView {
    let href = format!("/recipes/{}", row.id);
    let href = if filters.category.is_some() || filters.favourites || !filters.search.is_empty() {
        format!("{href}?return={}", encode_hex(&filters.href()))
    } else {
        href
    };
    let servings_key = if row.servings == 1 {
        "recipe.servings.one"
    } else {
        "recipe.servings.many"
    };
    let servings = t_with(servings_key, &[("count", &row.servings.to_string())]);
    let has_image = row.image.is_some();
    let loading = if first { "eager" } else { "lazy" };
    let image = row
        .image
        .map(|name| view! { <img src=format!("/images/{name}") alt="" loading=loading/> });
    let favourite = row.favourite.then(
        || view! { <span class="recipe-card-favourite">{t("recipe.favourite.badge")}</span> },
    );
    let rating = (row.rating_count > 0).then(|| rating_text(row.average, row.rating_count));
    view! {
        <section class="recipe-card">
            <a class="recipe-card-photo" href=href.clone() aria-label=row.title.clone()>
                {image}
                {(!has_image).then(|| view! { <span class="recipe-photo-fallback" aria-hidden="true"></span> })}
            </a>
            <div class="recipe-card-copy">
                <div class="recipe-card-meta">{favourite}</div>
                <h2><a href=href>{row.title}</a></h2>
                <p>{servings}</p>
                {rating.map(|text| view! { <p class="recipe-card-rating">{text}</p> })}
            </div>
        </section>
    }
}
