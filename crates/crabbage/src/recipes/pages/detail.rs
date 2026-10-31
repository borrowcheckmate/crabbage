//! The recipe page: the title, servings, ingredient lines and steps. The edit and delete actions
//! show only to the users who may use them, and delete asks first, which the `confirm` value in the
//! address asks for.

use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_query_map};

use super::rating_section::rating_section;
use super::stepper::servings_stepper;
use crate::i18n::{messages, t, t_with};
use crate::notice::{error_notice, failure, message};
use crate::recipes::cook::start_href;
use crate::recipes::duration::duration_text;
use crate::recipes::line_text::ingredient_parts;
use crate::recipes::read::get_recipe;
use crate::recipes::servings::{scale_lines, shown_servings};
use crate::recipes::types::{LineRow, RecipeDetail, StepRow};
use crate::recipes::write::DeleteRecipe;
use crate::route::route_id;

/// One recipe, loaded inside one suspense. Deleting it returns to the list.
#[component]
pub(crate) fn RecipePage() -> impl IntoView {
    let id = route_id();
    let query = use_query_map();
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
                let confirm = query.with(|map| map.get("confirm"));
                let servings = query.with(|map| map.get("servings"));
                let return_to = query.with(|map| map.get("return"));
                Suspend::new(async move { detail(id, confirm, servings, return_to, delete).await })
            }}
        </Suspense>
    }
}

async fn detail(
    id: Option<i64>,
    confirm: Option<String>,
    servings: Option<String>,
    return_to: Option<String>,
    delete: ServerAction<DeleteRecipe>,
) -> AnyView {
    let Some(id) = id else {
        return error_notice(t("catalog.error.missing"));
    };
    match get_recipe(id).await {
        Ok(recipe) => recipe_view(
            recipe,
            confirm.as_deref(),
            servings.as_deref(),
            return_to.as_deref(),
            delete,
        ),
        Err(error) => error_notice(&message(&error)),
    }
}

fn recipe_view(
    recipe: RecipeDetail,
    confirm: Option<&str>,
    requested: Option<&str>,
    return_to: Option<&str>,
    delete: ServerAction<DeleteRecipe>,
) -> AnyView {
    let RecipeDetail {
        id,
        title,
        servings,
        categories,
        lines,
        steps,
        image,
        can_edit,
        can_delete,
        average,
        rating_count,
        my_stars,
        favourite,
        ..
    } = recipe;
    let (edit_href, detail_href, delete_href) = recipe_links(id);
    let confirming = can_delete && confirm == Some("delete");
    let (shown, scaled) = scaled_lines(requested, servings, &lines);
    let cook = start_href(id, requested.map(|_| shown));
    let back_href = safe_return_to(return_to);
    let image_view = detail_photo(image, &title);
    view! {
        <article class="recipe-detail">
        <a class="recipe-back" href=back_href><svg aria-hidden="true" viewBox="0 0 24 24"><path d="m15 18-6-6 6-6M20 12H9"/></svg>{t("recipe.back")}</a>
        <header class="recipe-detail-header">
            <div class="recipe-detail-categories">{categories.into_iter().map(|category| view! { <span>{category.name}</span> }).collect_view()}{favourite.then(|| view! { <span class="recipe-detail-favorite">{t("recipe.favourite.badge")}</span> })}</div>
            <h1>{title.clone()}</h1>
        </header>
        <div class="recipe-detail-image">{image_view}</div>
        <div class="recipe-detail-columns">
            <section class="recipe-ingredients">
                <h2>{t("recipe.lines.heading")}</h2>
                {servings_stepper(id, shown, servings, return_to)}
                <ul>{scaled.iter().map(|line| ingredient_row(line, messages())).collect_view()}</ul>
            </section>
            <section class="recipe-method">
                <h2>{t("recipe.method.heading")}</h2>
                <ol>{steps.iter().map(step_item).collect_view()}</ol>
            </section>
        </div>
        <div class="recipe-detail-primary">
            <a class="button recipe-start" href=cook>{t("recipe.button.start_cooking")}</a>
        </div>
        <section class="recipe-secondary-actions">
            {rating_section(id, average, rating_count, my_stars, favourite)}
        </section>
        <div class="row recipe-edit-actions">
            {can_edit.then(|| view! { <a class="button" href=edit_href>{t("recipe.button.edit")}</a> })}
            {(can_delete && !confirming).then(|| view! { <a href=delete_href>{t("recipe.button.delete")}</a> })}
        </div>
        {confirming.then(|| delete_confirmation(id, &title, &detail_href, delete))}
        {move || delete.value().with(|value| failure(value.as_ref()))}
        </article>
    }
    .into_any()
}

fn recipe_links(id: i64) -> (String, String, String) {
    let detail = format!("/recipes/{id}");
    (
        format!("{detail}/edit"),
        detail.clone(),
        format!("{detail}?confirm=delete"),
    )
}

fn decode_hex(value: &str) -> Option<String> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    let (pairs, remainder) = value.as_bytes().as_chunks::<2>();
    if !remainder.is_empty() {
        return None;
    }
    let bytes = pairs
        .iter()
        .map(|pair| {
            let pair = std::str::from_utf8(pair).ok()?;
            u8::from_str_radix(pair, 16).ok()
        })
        .collect::<Option<Vec<_>>>()?;
    String::from_utf8(bytes).ok()
}

fn safe_return_to(value: Option<&str>) -> String {
    value
        .and_then(decode_hex)
        .filter(|href| *href == "/recipes" || href.starts_with("/recipes?"))
        .unwrap_or_else(|| "/recipes".to_owned())
}

fn scaled_lines(requested: Option<&str>, original: i64, lines: &[LineRow]) -> (i64, Vec<LineRow>) {
    let shown = shown_servings(requested, original);
    (shown, scale_lines(lines, original, shown))
}

fn detail_photo(image: Option<String>, title: &str) -> AnyView {
    image.map_or_else(
        || view! { <span class="recipe-photo-fallback" aria-hidden="true"></span> }.into_any(),
        |name| view! { <img class="recipe-detail-photo" src=format!("/images/{name}") alt=title.to_owned()/> }.into_any(),
    )
}

fn ingredient_row(line: &LineRow, messages: &crate::i18n::Messages) -> impl IntoView {
    let (name, quantity) = ingredient_parts(line, messages);
    view! {
        <li class="recipe-ingredient-row">
            <span>{name}</span>
            {(!quantity.is_empty()).then(|| view! { <span class="recipe-ingredient-quantity">{quantity}</span> })}
        </li>
    }
}

fn delete_confirmation(
    id: i64,
    title: &str,
    detail_href: &str,
    delete: ServerAction<DeleteRecipe>,
) -> AnyView {
    let question = t_with("recipe.confirm.delete", &[("name", title)]);
    view! {
        <section class="card">
            <p>{question}</p>
            <div class="row">
                <ActionForm action=delete>
                    <input type="hidden" name="id" value=id/>
                    <button class="danger">{t("recipe.button.delete")}</button>
                </ActionForm>
                <a href=detail_href.to_owned()>{t("admin.button.cancel")}</a>
            </div>
        </section>
    }
    .into_any()
}

fn step_item(step: &StepRow) -> impl IntoView {
    let minutes = step.duration_seconds.map(duration_text);
    view! {
        <li>
            <p>{step.text.clone()}</p>
            {minutes.map(|text| view! { <span class="recipe-step-duration">{text}</span> })}
        </li>
    }
}

#[cfg(test)]
mod tests {
    use super::safe_return_to;

    #[test]
    fn browse_context_decodes_and_rejects_external_destinations() {
        assert_eq!(
            safe_return_to(Some("2f726563697065733f713d62726f776e25323072696365")),
            "/recipes?q=brown%20rice"
        );
        assert_eq!(
            safe_return_to(Some("2f2f6576696c2e6578616d706c65")),
            "/recipes"
        );
        assert_eq!(safe_return_to(Some("xyz")), "/recipes");
    }
}
