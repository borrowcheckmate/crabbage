//! The import page: a form for the address of a recipe page, and under it the recipe read from that
//! page, in the recipe form, to review before saving. The address is the `url` value in the
//! address of the page, so reading a recipe is a plain GET.

use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_query_map};

use super::form::{Draft, DraftFields};
use crate::i18n::{t, t_with};
use crate::notice::{error_notice, failure, message};
use crate::recipes::import_fns::{SaveImport, ai_enabled, import_recipe, photo_preview};
use crate::recipes::ingredients::ingredient_names;
use crate::recipes::types::ImportPreview;

/// The page for importing a recipe. Saving opens the new recipe.
#[component]
pub(crate) fn RecipeImportPage() -> impl IntoView {
    let save = ServerAction::<SaveImport>::new();
    let navigate = use_navigate();
    Effect::new(move |_| {
        save.value().with(|value| {
            if let Some(Ok(id)) = value {
                navigate(&format!("/recipes/{id}"), NavigateOptions::default());
            }
        });
    });
    let query = use_query_map();
    view! {
        <h1>{t("import.heading")}</h1>
        <form method="get" action="/recipes/import">
            <label>
                {t("import.url_label")}
                <input type="url" name="url" required value=move || query.with(|map| map.get("url").unwrap_or_default())/>
            </label>
            <button>{t("import.button")}</button>
        </form>
        <Suspense fallback=|| ()>
            {move || {
                let (url, photos, error) = query.with(|map| {
                    (
                        map.get("url").unwrap_or_default().trim().to_owned(),
                        map.get("photos").unwrap_or_default(),
                        map.get("photo_error").unwrap_or_default(),
                    )
                });
                Suspend::new(async move { content(url, photos, error, save).await })
            }}
        </Suspense>
        {move || save.value().with(|value| failure(value.as_ref()))}
    }
}

async fn content(
    url: String,
    photos: String,
    error: String,
    save: ServerAction<SaveImport>,
) -> AnyView {
    let form = ai_enabled().await.unwrap_or(false).then(photo_form);
    let problem = match error.as_str() {
        "invalid" => Some(error_notice(t("import.photo_error.invalid"))),
        "ai" => Some(error_notice(t("import.photo_error.ai"))),
        _ => None,
    };
    view! { {form} {problem} {preview(url, photos, save).await} }.into_any()
}

fn photo_form() -> impl IntoView {
    view! {
        <form method="post" action="/recipes/import/photos" enctype="multipart/form-data">
            <label>
                {t("import.photos_label")}
                <input type="file" name="photo" accept="image/*" multiple required/>
            </label>
            <p>{t("import.photos_hint")}</p>
            <button>{t("import.photos_button")}</button>
        </form>
    }
}

async fn preview(url: String, photos: String, save: ServerAction<SaveImport>) -> AnyView {
    if url.is_empty() && photos.is_empty() {
        return ().into_any();
    }
    match load(url, photos).await {
        Ok((preview, names)) => review(preview, names, save),
        Err(error) => error_notice(&message(&error)),
    }
}

async fn load(url: String, photos: String) -> Result<(ImportPreview, Vec<String>), ServerFnError> {
    let preview = if photos.is_empty() {
        import_recipe(url).await?
    } else {
        photo_preview(photos).await?
    };
    Ok((preview, ingredient_names().await?))
}

fn review(preview: ImportPreview, names: Vec<String>, save: ServerAction<SaveImport>) -> AnyView {
    let ImportPreview {
        draft,
        image,
        source,
        unresolved,
        ai_error,
    } = preview;
    let draft = Draft::from_wire(&draft);
    let intro = t_with("import.source", &[("source", &source)]);
    let missing = (!unresolved.is_empty())
        .then(|| t_with("import.unresolved", &[("names", &unresolved.join(", "))]));
    let ai = ai_error.map(|text| view! { <p class="notice">{text}</p> });
    let photo = image.clone().map(|src| {
        view! {
            <section class="card">
                <h2>{t("import.photo")}</h2>
                <img class="photo" src=src alt=draft.title.get_untracked() referrerpolicy="no-referrer"/>
            </section>
        }
    });
    let image_field = image.map(|src| view! { <input type="hidden" name="image" value=src/> });
    view! {
        <p>{intro}</p>
        {missing.map(|text| view! { <p class="notice unresolved-list">{text}</p> })}
        {ai}
        <ActionForm action=save>
            <DraftFields draft names unresolved/>
            {photo}
            {image_field}
            <input type="hidden" name="draft" value=move || draft.json()/>
            <button>{t("recipe.button.save")}</button>
        </ActionForm>
    }
    .into_any()
}
