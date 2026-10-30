//! The ingredient catalog: create, rename, delete, aliases, store section and merging.

use leptos::form::ActionForm;
use leptos::prelude::*;

use super::notice::{error_notice, failure, message};
use crate::catalog::ingredients::{
    AddAlias, CreateIngredient, DeleteIngredient, MergeIngredients, RemoveAlias, RenameIngredient,
    SetIngredientSection, list_ingredients,
};
use crate::catalog::sections::list_sections;
use crate::catalog::{IngredientRow, SectionRow};
use crate::i18n::t;
use crate::user::current_user;

/// The actions a card of an ingredient can send.
#[derive(Clone, Copy)]
struct CardActions {
    rename: ServerAction<RenameIngredient>,
    delete: ServerAction<DeleteIngredient>,
    add_alias: ServerAction<AddAlias>,
    remove_alias: ServerAction<RemoveAlias>,
    set_section: ServerAction<SetIngredientSection>,
    merge: ServerAction<MergeIngredients>,
}

impl CardActions {
    fn new() -> Self {
        Self {
            rename: ServerAction::new(),
            delete: ServerAction::new(),
            add_alias: ServerAction::new(),
            remove_alias: ServerAction::new(),
            set_section: ServerAction::new(),
            merge: ServerAction::new(),
        }
    }
}

/// The ingredient catalog page. Admins see the forms and the rows, everyone else a message. The
/// data is loaded inside one suspense.
#[component]
pub(crate) fn IngredientsPage() -> impl IntoView {
    let create = ServerAction::<CreateIngredient>::new();
    let actions = CardActions::new();
    view! {
        <Suspense fallback=|| ()>
            {move || {
                let _versions = (
                    create.version().get(),
                    actions.rename.version().get(),
                    actions.delete.version().get(),
                    actions.add_alias.version().get(),
                    actions.remove_alias.version().get(),
                    actions.set_section.version().get(),
                    actions.merge.version().get(),
                );
                Suspend::new(async move { ingredient_page(create, actions).await })
            }}
        </Suspense>
    }
}

async fn ingredient_page(create: ServerAction<CreateIngredient>, actions: CardActions) -> AnyView {
    match current_user().await {
        Ok(user) if user.is_admin => {
            let rows = list_ingredients().await;
            let sections = list_sections().await;
            view! {
                <h1>{t("admin.ingredients.heading")}</h1>
                <ActionForm action=create>
                    <label>{t("admin.label.name")} <input name="name" required/></label>
                    <button>{t("admin.button.add")}</button>
                </ActionForm>
                {ingredient_failures(create, actions)}
                {ingredient_rows(Some(rows), Some(sections), actions)}
            }
            .into_any()
        }
        Ok(_) | Err(_) => error_notice(t("admin.not_allowed")),
    }
}

/// The messages of the last failed action of each kind on the ingredient page.
fn ingredient_failures(create: ServerAction<CreateIngredient>, actions: CardActions) -> AnyView {
    view! {
        {move || create.value().with(|value| failure(value.as_ref()))}
        {move || actions.rename.value().with(|value| failure(value.as_ref()))}
        {move || actions.delete.value().with(|value| failure(value.as_ref()))}
        {move || actions.add_alias.value().with(|value| failure(value.as_ref()))}
        {move || actions.remove_alias.value().with(|value| failure(value.as_ref()))}
        {move || actions.set_section.value().with(|value| failure(value.as_ref()))}
        {move || actions.merge.value().with(|value| failure(value.as_ref()))}
    }
    .into_any()
}

/// The rows of the ingredient list, or the message that explains why they are missing.
fn ingredient_rows(
    ingredients: Option<Result<Vec<IngredientRow>, ServerFnError>>,
    sections: Option<Result<Vec<SectionRow>, ServerFnError>>,
    actions: CardActions,
) -> AnyView {
    match (ingredients, sections) {
        (Some(Ok(rows)), Some(Ok(_))) if rows.is_empty() => {
            view! { <p>{t("admin.ingredients.empty")}</p> }.into_any()
        }
        (Some(Ok(rows)), Some(Ok(sections))) => rows
            .iter()
            .map(|row| {
                view! {
                    <IngredientCard
                        row=row.clone()
                        all=rows.clone()
                        sections=sections.clone()
                        actions=actions
                    />
                }
            })
            .collect_view()
            .into_any(),
        (Some(Err(error)), _) | (_, Some(Err(error))) => error_notice(&message(&error)),
        _ => ().into_any(),
    }
}

#[component]
fn IngredientCard(
    row: IngredientRow,
    all: Vec<IngredientRow>,
    sections: Vec<SectionRow>,
    actions: CardActions,
) -> impl IntoView {
    let id = row.id;
    let others: Vec<IngredientRow> = all.into_iter().filter(|other| other.id != id).collect();
    let section_id = row.section_id;
    let aliases = row.aliases.clone();
    view! {
        <section class="card">
            <h2>{row.name.clone()}</h2>
            {row.pending_review.then(|| view! { <p class="notice">{t("admin.ingredients.pending")}</p> })}
            <ActionForm action=actions.rename>
                <input type="hidden" name="id" value=id/>
                <label>{t("admin.label.name")} <input name="name" value=row.name.clone() required/></label>
                <button>{t("admin.button.rename")}</button>
            </ActionForm>
            <h3>{t("admin.label.aliases")}</h3>
            <ul>
                {aliases
                    .into_iter()
                    .map(|alias| view! {
                        <li>
                            <ActionForm action=actions.remove_alias>
                                <input type="hidden" name="ingredient_id" value=id/>
                                <input type="hidden" name="alias" value=alias.clone()/>
                                <span>{alias.clone()}</span>
                                <button class="secondary">{t("admin.button.remove")}</button>
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
            <ActionForm action=actions.set_section>
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
            {(!others.is_empty()).then(|| view! {
                <ActionForm action=actions.merge>
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
            })}
            <ActionForm action=actions.delete>
                <input type="hidden" name="id" value=id/>
                <button class="danger">{t("admin.button.delete")}</button>
            </ActionForm>
        </section>
    }
}
