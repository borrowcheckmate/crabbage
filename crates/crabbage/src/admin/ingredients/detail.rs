//! One ingredient's page: rename, alternative names, store section, merge and delete. Merge and
//! delete show a question first, which the `confirm` value in the address asks for.

use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_query_map};

use crate::catalog::ingredients::{
    AddAlias, DeleteIngredient, MergeIngredients, RemoveAlias, RenameIngredient,
    SetIngredientSection, list_ingredients,
};
use crate::catalog::sections::list_sections;
use crate::catalog::{IngredientRow, SectionRow};
use crate::i18n::{t, t_with};
use crate::notice::{error_notice, failure, message};
use crate::route::route_id;
use crate::user::current_user;

type IngredientDetailData = (
    Option<i64>,
    Option<String>,
    Result<crate::user::SignedInUser, ServerFnError>,
    Option<Result<Vec<IngredientRow>, ServerFnError>>,
    Option<Result<Vec<SectionRow>, ServerFnError>>,
);

/// The actions the ingredient page can send.
#[derive(Clone, Copy)]
struct IngredientActions {
    rename: ServerAction<RenameIngredient>,
    add_alias: ServerAction<AddAlias>,
    remove_alias: ServerAction<RemoveAlias>,
    set_section: ServerAction<SetIngredientSection>,
    delete: ServerAction<DeleteIngredient>,
    merge: ServerAction<MergeIngredients>,
}

/// The page of one ingredient. Deleting it, or merging it into another, returns to the list.
#[component]
pub(crate) fn IngredientPage() -> impl IntoView {
    let id = route_id();
    let query = use_query_map();
    let actions = IngredientActions {
        rename: ServerAction::new(),
        add_alias: ServerAction::new(),
        remove_alias: ServerAction::new(),
        set_section: ServerAction::new(),
        delete: ServerAction::new(),
        merge: ServerAction::new(),
    };
    let data = Resource::new(
        move || {
            (
                id(),
                query.with(|map| map.get("confirm")),
                actions.rename.version().get(),
                actions.add_alias.version().get(),
                actions.remove_alias.version().get(),
                actions.set_section.version().get(),
            )
        },
        |(id, confirm, ..)| load_ingredient_page(id, confirm),
    );
    let navigate = use_navigate();
    Effect::new(move |_| {
        let deleted = actions
            .delete
            .value()
            .with(|value| matches!(value, Some(Ok(()))));
        let merged = actions
            .merge
            .value()
            .with(|value| matches!(value, Some(Ok(()))));
        if deleted || merged {
            navigate("/admin/ingredients", NavigateOptions::default());
        }
    });
    view! {
        <Suspense fallback=|| view! { <p class="state-notice" role="status">{t("admin.loading")}</p> }>
            {move || {
                Suspend::new(async move { ingredient_page(data.await, actions) })
            }}
        </Suspense>
    }
}

async fn load_ingredient_page(
    id: Option<i64>,
    confirm: Option<String>,
) -> (
    Option<i64>,
    Option<String>,
    Result<crate::user::SignedInUser, ServerFnError>,
    Option<Result<Vec<IngredientRow>, ServerFnError>>,
    Option<Result<Vec<SectionRow>, ServerFnError>>,
) {
    let user = current_user().await;
    if !matches!(user, Ok(ref user) if user.is_admin) {
        return (id, confirm, user, None, None);
    }
    let rows = list_ingredients().await;
    let sections = if rows.is_ok() {
        Some(list_sections().await)
    } else {
        None
    };
    (id, confirm, user, Some(rows), sections)
}

fn ingredient_page(
    (id, confirm, user, rows, sections): IngredientDetailData,
    actions: IngredientActions,
) -> AnyView {
    if !matches!(user, Ok(user) if user.is_admin) {
        return error_notice(t("admin.not_allowed"));
    }
    let Some(id) = id else {
        return error_notice(t("catalog.error.missing"));
    };
    let rows = match rows.unwrap_or_else(|| Err(ServerFnError::new("Ingredients unavailable"))) {
        Ok(rows) => rows,
        Err(error) => return error_notice(&message(&error)),
    };
    let Some(row) = rows.iter().find(|row| row.id == id).cloned() else {
        return error_notice(t("catalog.error.missing"));
    };
    let sections = match sections.unwrap_or_else(|| Err(ServerFnError::new("Sections unavailable")))
    {
        Ok(sections) => sections,
        Err(error) => return error_notice(&message(&error)),
    };
    let others: Vec<IngredientRow> = rows.into_iter().filter(|other| other.id != id).collect();
    ingredient_view(row, others, sections, confirm.as_deref(), actions)
}

fn ingredient_view(
    row: IngredientRow,
    others: Vec<IngredientRow>,
    sections: Vec<SectionRow>,
    confirm: Option<&str>,
    actions: IngredientActions,
) -> AnyView {
    let id = row.id;
    let detail_href = format!("/admin/ingredients/{id}");
    let can_merge = !others.is_empty();
    let confirmation = match confirm {
        Some("delete") => delete_confirmation(id, &row.name, &detail_href, actions.delete),
        Some("merge") if can_merge => {
            merge_confirmation(id, &row.name, &detail_href, others, actions.merge)
        }
        Some(_) | None => action_links(&detail_href, can_merge),
    };
    view! {
        <section class="admin-page admin-detail">
        <p><a href="/admin/ingredients">{t("admin.ingredients.back")}</a></p>
        <h1>{row.name.clone()}</h1>
        {row.pending_review.then(|| view! { <p class="notice">{t("admin.ingredients.pending")}</p> })}
        {ingredient_failures(actions)}
        <section class="admin-row admin-detail-section">
        <ActionForm action=actions.rename>
            <input type="hidden" name="id" value=id/>
            <label>{t("admin.label.name")} <input name="name" value=row.name.clone() required/></label>
            <button>{t("admin.button.rename")}</button>
        </ActionForm>
        </section>
        {alias_section(id, row.aliases, actions)}
        {section_form(id, row.section_id, sections, actions.set_section)}
        {confirmation}
        </section>
    }
    .into_any()
}

/// The alternative names, each with a button to remove it, and the form that adds one.
fn alias_section(id: i64, aliases: Vec<String>, actions: IngredientActions) -> AnyView {
    view! {
        <section class="admin-row admin-detail-section">
        <h2>{t("admin.label.aliases")}</h2>
        <ul class="admin-sublist">
            {aliases
                .into_iter()
                .map(|alias| view! {
                    <li>
                        <ActionForm action=actions.remove_alias>
                            <input type="hidden" name="ingredient_id" value=id/>
                            <input type="hidden" name="alias" value=alias.clone()/>
                            <span>{alias.clone()}</span>
                            <button class="quiet">{t("admin.button.remove")}</button>
                        </ActionForm>
                    </li>
                })
                .collect_view()}
        </ul>
        <ActionForm action=actions.add_alias>
            <input type="hidden" name="ingredient_id" value=id/>
            <label>{t("admin.label.alias")} <input name="alias" required/></label>
            <button>{t("admin.button.add")}</button>
        </ActionForm>
        </section>
    }
    .into_any()
}

/// The store section choice, with none as the first option.
fn section_form(
    id: i64,
    section_id: Option<i64>,
    sections: Vec<SectionRow>,
    set_section: ServerAction<SetIngredientSection>,
) -> AnyView {
    view! {
        <section class="admin-row admin-detail-section">
        <ActionForm action=set_section>
            <input type="hidden" name="id" value=id/>
            <label>{t("admin.label.section")}
                <select name="section_id">
                    <option value="" selected=section_id.is_none()>{t("admin.label.no_section")}</option>
                    {sections
                        .iter()
                        .map(|section| view! {
                            <option
                                value=section.id.to_string()
                                selected=section_id == Some(section.id)
                            >
                                {section.name.clone()}
                            </option>
                        })
                        .collect_view()}
                </select>
            </label>
            <button>{t("admin.button.save")}</button>
        </ActionForm>
        </section>
    }
    .into_any()
}

/// The messages of the last failed action of each kind on the ingredient page.
fn ingredient_failures(actions: IngredientActions) -> AnyView {
    view! {
        {move || actions.rename.value().with(|value| failure(value.as_ref()))}
        {move || actions.add_alias.value().with(|value| failure(value.as_ref()))}
        {move || actions.remove_alias.value().with(|value| failure(value.as_ref()))}
        {move || actions.set_section.value().with(|value| failure(value.as_ref()))}
        {move || actions.delete.value().with(|value| failure(value.as_ref()))}
        {move || actions.merge.value().with(|value| failure(value.as_ref()))}
    }
    .into_any()
}

/// The links that ask for a confirmation. Merge is offered only when another ingredient exists.
fn action_links(detail_href: &str, can_merge: bool) -> AnyView {
    let delete_href = format!("{detail_href}?confirm=delete");
    let merge_href = format!("{detail_href}?confirm=merge");
    view! {
        <div class="row actions">
            <a href=delete_href>{t("admin.button.delete")}</a>
            {can_merge.then(|| view! { <a href=merge_href>{t("admin.button.merge")}</a> })}
        </div>
    }
    .into_any()
}

fn delete_confirmation(
    id: i64,
    name: &str,
    detail_href: &str,
    delete: ServerAction<DeleteIngredient>,
) -> AnyView {
    let question = t_with("admin.confirm.delete", &[("name", name)]);
    view! {
        <section class="admin-row admin-confirmation">
            <p>{question}</p>
            <div class="row">
                <ActionForm action=delete>
                    <input type="hidden" name="id" value=id/>
                    <button class="danger">{t("admin.button.delete")}</button>
                </ActionForm>
                <a href=detail_href.to_owned()>{t("admin.button.cancel")}</a>
            </div>
        </section>
    }
    .into_any()
}

fn merge_confirmation(
    id: i64,
    name: &str,
    detail_href: &str,
    others: Vec<IngredientRow>,
    merge: ServerAction<MergeIngredients>,
) -> AnyView {
    let question = t_with("admin.confirm.merge", &[("name", name)]);
    view! {
        <section class="admin-row admin-confirmation">
            <p>{question}</p>
            <ActionForm action=merge>
                <input type="hidden" name="from" value=id/>
                <label>{t("admin.label.merge_into")}
                    <select name="into">
                        {others
                            .iter()
                            .map(|other| view! {
                                <option value=other.id.to_string()>{other.name.clone()}</option>
                            })
                            .collect_view()}
                    </select>
                </label>
                <button>{t("admin.button.merge")}</button>
            </ActionForm>
            <a href=detail_href.to_owned()>{t("admin.button.cancel")}</a>
        </section>
    }
    .into_any()
}
