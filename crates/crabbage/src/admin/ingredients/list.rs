//! The ingredient list: the form for a new ingredient, a search over names and alternative names,
//! and one row per ingredient that links to its page.

use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::catalog::ingredients::{CreateIngredient, list_ingredients};
use crate::catalog::sections::list_sections;
use crate::catalog::{IngredientRow, SectionRow};
use crate::i18n::t;
use crate::notice::{error_notice, failure, message};
use crate::user::current_user;

type IngredientListRows = (
    Result<Vec<IngredientRow>, ServerFnError>,
    Result<Vec<SectionRow>, ServerFnError>,
);
type IngredientListData = (
    String,
    Result<crate::user::SignedInUser, ServerFnError>,
    Option<IngredientListRows>,
);

/// The ingredient list page. Admins see the add form, the search and the rows, everyone else a
/// message. The data is loaded inside one suspense, and the search reads `q` from the address.
#[component]
pub(crate) fn IngredientsPage() -> impl IntoView {
    let create = ServerAction::<CreateIngredient>::new();
    let query = use_query_map();
    let data = Resource::new(
        move || {
            (
                query.with(|map| map.get("q")).unwrap_or_default(),
                create.version().get(),
            )
        },
        |(search, _)| load_ingredient_page(search),
    );
    view! {
        <Suspense fallback=|| view! { <p class="state-notice" role="status">{t("admin.loading")}</p> }>
            {move || {
                Suspend::new(async move { ingredient_page(data.await, create) })
            }}
        </Suspense>
    }
}

async fn load_ingredient_page(
    search: String,
) -> (
    String,
    Result<crate::user::SignedInUser, ServerFnError>,
    Option<(
        Result<Vec<IngredientRow>, ServerFnError>,
        Result<Vec<SectionRow>, ServerFnError>,
    )>,
) {
    let user = current_user().await;
    let rows = if user.as_ref().is_ok_and(|user| user.is_admin) {
        Some((list_ingredients().await, list_sections().await))
    } else {
        None
    };
    (search, user, rows)
}

fn ingredient_page(
    (search, user, rows): IngredientListData,
    create: ServerAction<CreateIngredient>,
) -> AnyView {
    match user {
        Ok(user) if user.is_admin => {
            let (rows, sections) = rows.unwrap_or_else(|| (Ok(Vec::new()), Ok(Vec::new())));
            view! {
                <section class="admin-page">
                <h1>{t("admin.ingredients.heading")}</h1>
                <section class="admin-card admin-create"><ActionForm action=create>
                    <label>{t("admin.label.name")} <input name="name" required/></label>
                    <button>{t("admin.button.add")}</button>
                </ActionForm></section>
                {move || create.value().with(|value| failure(value.as_ref()))}
                <form class="admin-search" method="get" action="/admin/ingredients">
                    <label>{t("admin.label.search")} <input type="search" name="q" value=search.clone()/></label>
                    <button>{t("admin.button.search")}</button>
                </form>
                {ingredient_rows(rows, sections, &search)}
                </section>
            }
            .into_any()
        }
        Ok(_) | Err(_) => error_notice(t("admin.not_allowed")),
    }
}

/// The rows that match `search`, or the message that explains why there are none.
fn ingredient_rows(
    rows: Result<Vec<IngredientRow>, ServerFnError>,
    sections: Result<Vec<SectionRow>, ServerFnError>,
    search: &str,
) -> AnyView {
    match (rows, sections) {
        (Ok(rows), Ok(_)) if rows.is_empty() => {
            view! { <p>{t("admin.ingredients.empty")}</p> }.into_any()
        }
        (Ok(rows), Ok(sections)) => {
            let needle = search.trim().to_lowercase();
            let matching: Vec<IngredientRow> = rows
                .into_iter()
                .filter(|row| matches_search(row, &needle))
                .collect();
            if matching.is_empty() {
                return view! { <p>{t("admin.ingredients.no_match")}</p> }.into_any();
            }
            view! {
                <ul class="ingredient-list">
                    {matching
                        .iter()
                        .map(|row| ingredient_item(row, &sections))
                        .collect_view()}
                </ul>
            }
            .into_any()
        }
        (Err(error), _) | (_, Err(error)) => error_notice(&message(&error)),
    }
}

/// Whether `row` matches the lowercase `needle` by its name or by one of its alternative names.
fn matches_search(row: &IngredientRow, needle: &str) -> bool {
    row.name.to_lowercase().contains(needle)
        || row
            .aliases
            .iter()
            .any(|alias| alias.to_lowercase().contains(needle))
}

/// A list row: one link with the name, and the section and alternative-name count underneath.
fn ingredient_item(row: &IngredientRow, sections: &[SectionRow]) -> impl IntoView {
    let href = format!("/admin/ingredients/{}", row.id);
    let section = sections
        .iter()
        .find(|section| Some(section.id) == row.section_id)
        .map(|section| section.name.clone());
    let aliases = (!row.aliases.is_empty())
        .then(|| format!("{}: {}", t("admin.label.aliases"), row.aliases.len()));
    let pending = row.pending_review.then(|| {
        view! { <span class="badge">{t("admin.ingredients.pending")}</span> }
    });
    view! {
        <li>
            <a class="ingredient-row" href=href>
                <span class="ingredient-name">{row.name.clone()} {pending}</span>
                <span class="ingredient-meta">
                    {section.map(|name| view! { <span>{name}</span> })}
                    {aliases.map(|aliases| view! { <span>{aliases}</span> })}
                </span>
                <svg class="row-chevron" aria-hidden="true" viewBox="0 0 24 24"><path d="m9 18 6-6-6-6"/></svg>
            </a>
        </li>
    }
}
