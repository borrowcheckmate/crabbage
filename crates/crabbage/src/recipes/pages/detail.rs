//! The recipe page: the title, servings, ingredient lines and steps. The edit and delete buttons
//! show only to the users who may use them.

use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;

use crate::i18n::{messages, t};
use crate::notice::{error_notice, failure, message};
use crate::recipes::line_text::line_text;
use crate::recipes::read::get_recipe;
use crate::recipes::types::{RecipeDetail, StepRow};
use crate::recipes::write::DeleteRecipe;
use crate::route::route_id;

/// One recipe, loaded inside one suspense. Deleting it returns to the list.
#[component]
pub(crate) fn RecipePage() -> impl IntoView {
    let id = route_id();
    let delete = ServerAction::<DeleteRecipe>::new();
    let navigate = use_navigate();
    Effect::new(move |_| {
        delete.value().with(|value| {
            if matches!(value, Some(Ok(()))) {
                navigate("/recipes", NavigateOptions::default());
            }
        });
    });
    view! {
        <Suspense fallback=|| ()>
            {move || {
                let id = id();
                Suspend::new(async move { detail(id, delete).await })
            }}
        </Suspense>
    }
}

async fn detail(id: Option<i64>, delete: ServerAction<DeleteRecipe>) -> AnyView {
    let Some(id) = id else {
        return error_notice(t("catalog.error.missing"));
    };
    match get_recipe(id).await {
        Ok(recipe) => recipe_view(recipe, delete),
        Err(error) => error_notice(&message(&error)),
    }
}

fn recipe_view(recipe: RecipeDetail, delete: ServerAction<DeleteRecipe>) -> AnyView {
    let RecipeDetail {
        id,
        title,
        servings,
        owner_name,
        lines,
        steps,
        can_edit,
        can_delete,
        ..
    } = recipe;
    let edit_href = format!("/recipes/{id}/edit");
    let servings = format!("{}: {servings}", t("recipe.servings"));
    let owner = format!("{}: {owner_name}", t("recipe.added_by"));
    view! {
        <h1>{title}</h1>
        <p>{servings}</p>
        <p>{owner}</p>
        <h2>{t("recipe.lines.heading")}</h2>
        <ul>{lines.iter().map(|line| view! { <li>{line_text(line, messages())}</li> }).collect_view()}</ul>
        <h2>{t("recipe.steps.heading")}</h2>
        <ol>{steps.iter().map(step_item).collect_view()}</ol>
        <div class="row">
            {can_edit.then(|| view! { <a class="button" href=edit_href>{t("recipe.button.edit")}</a> })}
            {can_delete.then(|| view! {
                <ActionForm action=delete>
                    <input type="hidden" name="id" value=id/>
                    <button class="danger">{t("recipe.button.delete")}</button>
                </ActionForm>
            })}
        </div>
        {move || delete.value().with(|value| failure(value.as_ref()))}
    }
    .into_any()
}

fn step_item(step: &StepRow) -> impl IntoView {
    let minutes = step
        .duration_seconds
        .map(|seconds| format!("{} {}", (seconds + 59) / 60, t("unit.minutes")));
    view! {
        <li>
            <p>{step.text.clone()}</p>
            {minutes.map(|text| view! { <p>{text}</p> })}
        </li>
    }
}
