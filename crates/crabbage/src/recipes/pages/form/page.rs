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

/// The page for a new recipe. Saving opens the recipe.
#[component]
pub(crate) fn RecipeNewPage() -> impl IntoView {
    let create = ServerAction::<CreateRecipe>::new();
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
        <Suspense fallback=|| ()>
            {move || Suspend::new(async move { new_form(create).await })}
        </Suspense>
        {move || create.value().with(|value| failure(value.as_ref()))}
    }
}

async fn new_form(create: ServerAction<CreateRecipe>) -> AnyView {
    match load_new_form().await {
        Ok((names, categories)) => {
            let draft = Draft::blank();
            view! {
                <ActionForm action=create>
                    <DraftFields draft names/>
                    <CategoryBoxes draft categories/>
                    <input type="hidden" name="draft" value=move || draft.json()/>
                    <button>{t("recipe.button.save")}</button>
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
        <Suspense fallback=|| ()>
            {move || {
                let id = id();
                Suspend::new(async move { edit_form(id, save).await })
            }}
        </Suspense>
        {move || save.value().with(|value| failure(value.as_ref()))}
    }
}

async fn edit_form(id: Option<i64>, save: ServerAction<UpdateRecipe>) -> AnyView {
    let Some(id) = id else {
        return error_notice(t("catalog.error.missing"));
    };
    match load_edit_form(id, save).await {
        Ok(view) => view,
        Err(error) => error_notice(&message(&error)),
    }
}

async fn load_edit_form(
    id: i64,
    save: ServerAction<UpdateRecipe>,
) -> Result<AnyView, ServerFnError> {
    let recipe = get_recipe(id).await?;
    if !recipe.can_edit {
        return Ok(error_notice(t("recipe.not_allowed")));
    }
    let names = ingredient_names().await?;
    let users = list_users().await?;
    let categories = list_recipe_categories().await?;
    let draft = Draft::of(&recipe);
    let can_manage_editors = recipe.can_manage_editors;
    let photo = photo_card(recipe.id, &recipe.title, recipe.image.as_deref());
    let editors = recipe.editors;
    Ok(view! {
        <ActionForm action=save>
            <input type="hidden" name="id" value=recipe.id/>
            <DraftFields draft names/>
            <CategoryBoxes draft categories/>
            <input type="hidden" name="draft" value=move || draft.json()/>
            <button>{t("recipe.button.save")}</button>
        </ActionForm>
        {photo}
        {can_manage_editors.then(|| view! {
            <EditorsSection recipe_id=recipe.id editors users/>
        })}
    }
    .into_any())
}
