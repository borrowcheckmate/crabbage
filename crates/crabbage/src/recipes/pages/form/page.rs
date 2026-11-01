//! The pages for a new recipe and for changing one. Both send the whole form as one JSON draft.

use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;

use super::draft::Draft;
use super::fields::{CategoryBoxes, DraftFields};
use crate::catalog::CategoryRow;
use crate::i18n::t;
use crate::notice::{error_notice, failure, message};
use crate::recipes::categories::list_recipe_categories;
use crate::recipes::editors::list_users;
use crate::recipes::ingredients::ingredient_names;
use crate::recipes::pages::editor_list::EditorsSection;
use crate::recipes::pages::photo_card::photo_card;
use crate::recipes::read::get_recipe;
use crate::recipes::write::{CreateRecipe, UpdateRecipe};
use crate::route::route_id;

type EditFormData = Result<
    Option<(
        crate::recipes::types::RecipeDetail,
        Vec<String>,
        Vec<crate::recipes::types::PersonRow>,
        Vec<CategoryRow>,
    )>,
    ServerFnError,
>;

/// The page for a new recipe. Saving opens the recipe.
#[component]
pub(crate) fn RecipeNewPage() -> impl IntoView {
    let create = ServerAction::<CreateRecipe>::new();
    let data = Resource::new(|| (), |()| load_new_form());
    let navigate = use_navigate();
    Effect::new(move |_| {
        create.value().with(|value| {
            if let Some(Ok(id)) = value {
                navigate(&format!("/recipes/{id}"), NavigateOptions::default());
            }
        });
    });
    view! {
        <h1>{t("recipe.new_heading")}</h1>
        <Suspense fallback=|| view! { <p class="notice" role="status">{t("recipe.form.loading")}</p> }>
            {move || Suspend::new(async move { new_form(data.await, create) })}
        </Suspense>
        {move || create.value().with(|value| failure(value.as_ref()))}
    }
}

fn new_form(
    result: Result<(Vec<String>, Vec<CategoryRow>), ServerFnError>,
    create: ServerAction<CreateRecipe>,
) -> AnyView {
    match result {
        Ok((names, categories)) => {
            let draft = Draft::blank();
            view! {
                <ActionForm action=create attr:class="recipe-form">
                    <DraftFields draft names/>
                    <CategoryBoxes draft categories/>
                    <input type="hidden" name="draft" value=move || draft.json()/>
                    <button disabled=move || create.pending().get() aria-busy=move || create.pending().get().to_string()>
                        {move || if create.pending().get() { t("recipe.form.saving") } else { t("recipe.button.save") }}
                    </button>
                </ActionForm>
            }
            .into_any()
        }
        Err(error) => error_notice(&message(&error)),
    }
}

async fn load_new_form() -> Result<(Vec<String>, Vec<CategoryRow>), ServerFnError> {
    Ok((ingredient_names().await?, list_recipe_categories().await?))
}

/// The page for changing a recipe. Saving opens the recipe. Owners and admins also see the editors.
#[component]
pub(crate) fn RecipeEditPage() -> impl IntoView {
    let id = route_id();
    let save = ServerAction::<UpdateRecipe>::new();
    let data = Resource::new(id, |id| async move {
        match id {
            Some(id) => Some(load_edit_form(id).await),
            None => None,
        }
    });
    let navigate = use_navigate();
    Effect::new(move |_| {
        save.value().with(|value| {
            if let (Some(Ok(())), Some(id)) = (value, id()) {
                navigate(&format!("/recipes/{id}"), NavigateOptions::default());
            }
        });
    });
    view! {
        <h1>{t("recipe.edit_heading")}</h1>
        <Suspense fallback=|| view! { <p class="notice" role="status">{t("recipe.form.loading")}</p> }>
            {move || {
                Suspend::new(async move { edit_form(data.await, save) })
            }}
        </Suspense>
        {move || save.value().with(|value| failure(value.as_ref()))}
    }
}

fn edit_form(data: Option<EditFormData>, save: ServerAction<UpdateRecipe>) -> AnyView {
    let Some(data) = data else {
        return error_notice(t("catalog.error.missing"));
    };
    let (recipe, names, users, categories) = match data {
        Ok(Some(data)) => data,
        Ok(None) => return error_notice(t("recipe.not_allowed")),
        Err(error) => return error_notice(&message(&error)),
    };
    let draft = Draft::of(&recipe);
    let can_manage_editors = recipe.can_manage_editors;
    let photo = photo_card(recipe.id, &recipe.title, recipe.image.as_deref());
    let editors = recipe.editors;
    view! {
        <ActionForm action=save attr:class="recipe-form">
            <input type="hidden" name="id" value=recipe.id/>
            <DraftFields draft names/>
            <CategoryBoxes draft categories/>
            <input type="hidden" name="draft" value=move || draft.json()/>
            <button disabled=move || save.pending().get() aria-busy=move || save.pending().get().to_string()>
                {move || if save.pending().get() { t("recipe.form.saving") } else { t("recipe.button.save") }}
            </button>
        </ActionForm>
        {photo}
        {can_manage_editors.then(|| view! { <EditorsSection recipe_id=recipe.id editors users/> })}
    }.into_any()
}

async fn load_edit_form(id: i64) -> EditFormData {
    let recipe = get_recipe(id).await?;
    if !recipe.can_edit {
        return Ok(None);
    }
    let names = ingredient_names().await?;
    let users = list_users().await?;
    let categories = list_recipe_categories().await?;
    Ok(Some((recipe, names, users, categories)))
}
