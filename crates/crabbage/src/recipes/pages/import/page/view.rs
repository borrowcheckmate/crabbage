//! The recipe import source, pending, review and recovery states.

use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_query_map};

use super::super::super::form::Draft;
use crate::i18n::{t, t_with};
use crate::notice::{failure, message};
use crate::recipes::import_fns::{SaveImport, ai_enabled, import_recipe, photo_preview};
use crate::recipes::ingredients::ingredient_names;
use crate::recipes::types::ImportPreview;

#[path = "fields.rs"]
mod fields;

use fields::import_fields;

type ImportReviewResult = Option<Result<(ImportPreview, Vec<String>), ServerFnError>>;
type ImportPageData = (
    String,
    String,
    bool,
    Result<bool, ServerFnError>,
    ImportReviewResult,
);

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
    let data = Resource::new(
        move || {
            query.with(|map| {
                (
                    map.get("url").unwrap_or_default().trim().to_owned(),
                    map.get("photos").unwrap_or_default(),
                    map.get("text").unwrap_or_default(),
                )
            })
        },
        |(url, photos, text)| load_content(url, photos, text),
    );
    view! {
        <Suspense fallback=|| view! { <p class="notice" role="status">{t("import.pending")}</p> }>
            {move || {
                let (error, source) = query.with(|map| (
                    map.get("photo_error")
                        .or_else(|| map.get("text_error"))
                        .unwrap_or_default(),
                    map.get("source").unwrap_or_default(),
                ));
                let source_url = query.with(|map| map.get("source_url").unwrap_or_default());
                Suspend::new(async move { content(data.await, &error, &source, source_url, save) })
            }}
        </Suspense>
        {move || save.value().with(|value| failure(value.as_ref()))}
    }
}

async fn load_content(url: String, photos: String, text: String) -> ImportPageData {
    let from_text = !text.is_empty();
    let photo_available = ai_enabled().await;
    let review = if !text.is_empty() {
        Some(
            async {
                let preview = photo_preview(text.clone()).await?;
                Ok((preview, ingredient_names().await?))
            }
            .await,
        )
    } else if !url.is_empty() || !photos.is_empty() {
        Some(load(url.clone(), photos.clone()).await)
    } else {
        None
    };
    (url, photos, from_text, photo_available, review)
}

fn content(
    (url, photos, from_text, photo_available, loaded): ImportPageData,
    error: &str,
    source: &str,
    source_url: String,
    save: ServerAction<SaveImport>,
) -> AnyView {
    if let Some(loaded) = loaded {
        let (preview, names) = match loaded {
            Ok(value) => value,
            Err(error) => {
                return import_failure(&message(&error), &url, from_text || !photos.is_empty());
            }
        };
        let review_url = if from_text { source_url } else { url };
        return review(preview, names, review_url, photos, from_text, save);
    }
    source_page(source, photo_available.unwrap_or(false), error)
}

fn source_page(source: &str, photo_available: bool, error: &str) -> AnyView {
    let website = source == "website";
    let problem = match error {
        "invalid" => Some(t("import.photo_error.invalid")),
        "text_invalid" => Some(t("import.text_error.invalid")),
        "ai" => Some(t("import.photo_error.ai")),
        _ => None,
    };
    view! {
        <a class="import-back" href="/recipes"><svg aria-hidden="true" viewBox="0 0 24 24"><path d="m15 18-6-6 6-6M9 12h12"/></svg>{t("import.back")}</a>
        <ImportProgress current=0/>
        <header class="import-heading">
            <h1>{t("import.heading")}</h1>
            <p>{t("import.choose_help")}</p>
        </header>
        <nav class="import-tabs" aria-label=t("import.source_choices")>
            <a class:active=source != "website" && source != "text" href="/recipes/import?source=photos" aria-current=(source != "website" && source != "text").then_some("page")>
                <svg aria-hidden="true" viewBox="0 0 24 24"><path d="M14 4h-4l-2 2H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-3z"/><circle cx="12" cy="12" r="3"/></svg>{t("import.source.photos")}
            </a>
            <a class:active=website href="/recipes/import?source=website" aria-current=website.then_some("page")>
                <svg aria-hidden="true" viewBox="0 0 24 24"><path d="M10 13a5 5 0 0 0 7.1 0l3-3a5 5 0 0 0-7.1-7.1l-1.7 1.7M14 11a5 5 0 0 0-7.1 0l-3 3A5 5 0 0 0 7 21.1l1.7-1.7"/></svg>{t("import.source.url")}
            </a>
            <a class:active=source == "text" href="/recipes/import?source=text" aria-current=(source == "text").then_some("page")>
                <svg aria-hidden="true" viewBox="0 0 24 24"><rect x="5" y="3" width="14" height="18" rx="2"/><path d="M9 3.5h6M9 9h6M9 13h6M9 17h4"/></svg>{t("import.source.text")}
            </a>
        </nav>
        {if website { website_form().into_any() } else if source == "text" { text_form().into_any() } else if photo_available { photo_source().into_any() } else { unavailable_notice() }}
        {problem.map(|text| view! { <p class="notice error" role="alert">{text}</p> })}
        <a class="import-manual" href="/recipes/new"><svg aria-hidden="true" viewBox="0 0 24 24"><path d="m15 5 4 4M4 20l4-.8L19 8a2.8 2.8 0 0 0-4-4L4 15z"/></svg>{t("import.manual")}</a>
    }.into_any()
}

fn text_form() -> impl IntoView {
    view! {
        <form method="post" action="/recipes/import/text" class="import-source-card">
            <h2>{t("import.text_heading")}</h2>
            <label>{t("import.text_label")}<textarea name="text" required rows="12"></textarea></label>
            <label>{t("import.source_url_label")}<input type="url" name="source_url" placeholder="https://"/></label>
            <button class="import-primary">{t("import.text_button")}</button>
        </form>
    }
}

fn website_form() -> impl IntoView {
    view! {
        <form method="get" action="/recipes/import" class="import-source-card">
            <input type="hidden" name="source" value="website"/>
            <h2>{t("import.website_heading")}</h2>
            <label>{t("import.url_label")}<input type="url" name="url" required placeholder="https://"/></label>
            <p>{t("import.url_hint")}</p>
            <button class="import-primary"><svg aria-hidden="true" viewBox="0 0 24 24"><path d="M5 12h14m-6-6 6 6-6 6"/></svg>{t("import.button")}</button>
        </form>
    }
}

fn photo_source() -> impl IntoView {
    view! {
        <section class="import-source-card">
            <h2>{t("import.photos_heading")}</h2>
            <p>{t("import.photos_hint")}</p>
            <form method="post" action="/recipes/import/photos" enctype="multipart/form-data" class="import-file-form">
                <label class="import-file import-camera"><svg aria-hidden="true" viewBox="0 0 24 24"><path d="M14 4h-4L8 6H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-3z"/><circle cx="12" cy="12" r="3"/></svg>{t("import.take_photo")}
                    <input type="file" name="photo" accept="image/jpeg,image/png,image/webp" capture="environment" required/>
                </label>
            </form>
            <form method="post" action="/recipes/import/photos" enctype="multipart/form-data" class="import-file-form">
                <label class="import-file"><svg aria-hidden="true" viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="m21 15-5-5L5 21"/></svg>{t("import.choose_photos")}
                    <input type="file" name="photo" accept="image/jpeg,image/png,image/webp" multiple required/>
                </label>
            </form>
            <p class="import-file-hint">{t("import.photos_limits")}</p>
            <p class="import-config-note">{t("import.ai_service")}</p>
        </section>
        <section class="import-source-advice"><h3>{t("import.photo_tips_heading")}</h3><p>{t("import.photo_tips")}</p></section>
    }
}

fn unavailable_notice() -> AnyView {
    view! {
        <section class="import-unavailable" role="status">
            <h2>{t("import.unavailable_heading")}</h2>
            <p>{t("import.photos.unavailable")}</p>
            <a href="/recipes/import?source=website">{t("import.source.url")}</a>
        </section>
    }
    .into_any()
}

async fn load(url: String, photos: String) -> Result<(ImportPreview, Vec<String>), ServerFnError> {
    let preview = if photos.is_empty() {
        import_recipe(url).await?
    } else {
        photo_preview(photos).await?
    };
    Ok((preview, ingredient_names().await?))
}

fn review(
    preview: ImportPreview,
    names: Vec<String>,
    url: String,
    photos: String,
    from_text: bool,
    save: ServerAction<SaveImport>,
) -> AnyView {
    let ImportPreview {
        draft,
        image,
        source,
        unresolved,
        ai_error,
        photo_preview,
    } = preview;
    let draft = Draft::from_wire(&draft);
    let source_name = if from_text {
        t("import.source.text").to_owned()
    } else if photos.is_empty() {
        t("import.source.url").to_owned()
    } else {
        source
    };
    let from_photos = !photos.is_empty();
    let source_panel =
        review_source_panel(source_name.clone(), url.clone(), image.clone(), from_photos);
    let form = review_form(ReviewForm {
        save,
        draft,
        names,
        unresolved,
        ai_error,
        source: source_name,
        url,
        photos,
        image,
        photo_preview,
        from_photos,
    });
    view! {
        <ReviewHeader/>
        <div class="import-review-layout">{source_panel}{form}</div>
    }
    .into_any()
}

struct ReviewForm {
    save: ServerAction<SaveImport>,
    draft: Draft,
    names: Vec<String>,
    unresolved: Vec<String>,
    ai_error: Option<String>,
    source: String,
    url: String,
    photos: String,
    image: Option<String>,
    photo_preview: Option<String>,
    from_photos: bool,
}

#[component]
fn ReviewHeader() -> impl IntoView {
    view! {
        <a class="import-back" href="/recipes/import"><svg aria-hidden="true" viewBox="0 0 24 24"><path d="m15 18-6-6 6-6M9 12h12"/></svg>{t("import.back_review")}</a>
        <ImportProgress current=2/>
        <header class="import-heading"><h1>{t("import.review.heading")}</h1><p>{t("import.review.help")}</p></header>
    }
}

fn review_source_panel(
    source: String,
    url: String,
    image: Option<String>,
    from_photos: bool,
) -> AnyView {
    view! { <aside class="import-source-desktop"><SourceCard source url image from_photos/></aside> }.into_any()
}

fn review_form(form: ReviewForm) -> AnyView {
    let ReviewForm {
        save,
        draft,
        names,
        unresolved,
        ai_error,
        source,
        url,
        photos,
        image,
        photo_preview,
        from_photos,
    } = form;
    let unresolved_for_note = unresolved.clone();
    let disclosure = review_source_disclosure(source, url, image.clone(), from_photos);
    let notices = review_notices(&unresolved, ai_error);
    let fields = import_fields(draft, names, unresolved);
    let photos = review_photo_fields(image, photo_preview, photos, draft);
    view! {
        <ActionForm action=save attr:class="import-review-form">
            {disclosure}
            {notices}
            {fields}
            {photos}
            {(!unresolved_for_note.is_empty()).then(|| view! { <p class="import-catalog-note">{t("import.catalog_note")}</p> })}
            <ReviewSaveButton save/>
        </ActionForm>
    }.into_any()
}

fn review_source_disclosure(
    source: String,
    url: String,
    image: Option<String>,
    from_photos: bool,
) -> AnyView {
    let summary = if from_photos {
        t("import.source_disclosure_photo")
    } else {
        t("import.source_disclosure_url")
    };
    view! { <details class="import-source-mobile"><summary>{summary}</summary><SourceCard source url image from_photos/></details> }.into_any()
}

fn review_photo_fields(
    image: Option<String>,
    photo_preview: Option<String>,
    photos: String,
    draft: Draft,
) -> AnyView {
    let crop = photo_preview.map(|src| view! { <section class="import-crop"><h3>{t("import.photo")}</h3><img class="photo" src=src alt=draft.title.get_untracked()/><label><input type="checkbox" name="keep_photo" value="true" checked/>{t("import.keep_photo")}</label></section> });
    let image_field = image.map(|src| view! { <input type="hidden" name="image" value=src/> });
    view! { {crop} {image_field} <input type="hidden" name="photo_token" value=photos/> <input type="hidden" name="draft" value=move || draft.json()/> }.into_any()
}

#[component]
fn ReviewSaveButton(save: ServerAction<SaveImport>) -> impl IntoView {
    view! {
        <button class="import-primary import-save" disabled=move || save.pending().get() aria-busy=move || save.pending().get()>
            <svg aria-hidden="true" viewBox="0 0 24 24"><path d="m5 12 4 4L19 6"/></svg>{move || if save.pending().get() { t("recipe.form.saving") } else { t("import.save") }}
        </button>
    }
}

fn review_notices(unresolved: &[String], ai_error: Option<String>) -> AnyView {
    let ai_off = ai_error
        .as_deref()
        .filter(|text| *text == t("import.ai_off"))
        .map(str::to_owned);
    let warning = (!unresolved.is_empty()).then(|| {
        let attention_count = if unresolved.len() == 1 {
            t("import.attention.one").to_owned()
        } else {
            t_with(
                "import.attention.many",
                &[("count", &unresolved.len().to_string())],
            )
        };
        let names = unresolved.join(", ");
        let message = t_with("import.unresolved", &[("names", &names)]);
        view! { <div class="import-warning" role="status"><svg aria-hidden="true" viewBox="0 0 24 24"><path d="M12 3 2 21h20L12 3z"/><path d="M12 9v4m0 4h.01"/></svg><div><strong>{attention_count}</strong><p>{message}</p>{ai_off.map(|note| view! { <p>{note}</p> })}</div></div> }.into_any()
    });
    let ai_note = ai_error
        .filter(|text| !text.eq(&t("import.ai_off")))
        .map(|text| view! { <p class="import-ai-note" role="status">{text}</p> }.into_any());
    view! { {warning} {ai_note} }.into_any()
}

#[component]
fn ImportProgress(current: u8) -> impl IntoView {
    let stages = [
        "import.stage.source",
        "import.stage.read",
        "import.stage.review",
    ];
    view! {
        <ol class="import-progress" aria-label=t("import.progress_label")>
            {stages.into_iter().enumerate().map(|(index, key)| view! {
            <li class:current=index == usize::from(current) class:complete=index < usize::from(current) aria-current=(index == usize::from(current)).then_some("step")>
                    <span>{if index < usize::from(current) { view! { <svg aria-hidden="true" viewBox="0 0 24 24"><path d="m5 12 4 4L19 6"/></svg> }.into_any() } else { (index + 1).to_string().into_any() }}</span>{t(key)}
                </li>
            }).collect_view()}
        </ol>
    }
}

#[component]
fn SourceCard(
    source: String,
    url: String,
    image: Option<String>,
    from_photos: bool,
) -> impl IntoView {
    let source_link = (!url.is_empty()).then(
        || view! { <a href=url target="_blank" rel="noreferrer">{t("import.open_source")}</a> },
    );
    let photo = image.map(|src| view! { <img class="import-source-image" src=src alt=t("import.photo") referrerpolicy="no-referrer"/> });
    view! {
        <section class="import-source-card import-source-card-review">
            <p class="import-source-label">{t_with("import.source_label", &[("source", &source)])}</p>
            {photo}
            {source_link}
            {from_photos.then(|| view! { <p class="import-source-limited">{t("import.original_pages_unavailable")}</p> })}
        </section>
    }
}

pub(super) fn format_amount(amount: &str, unit: &str) -> String {
    let unit = crate::recipes::unit_text(crate::i18n::messages(), unit.trim());
    match (amount.trim(), unit.trim()) {
        ("", "") => t("import.unspecified").to_owned(),
        ("", unit) => unit.to_owned(),
        (amount, "") => amount.to_owned(),
        (amount, unit) => format!("{amount} {unit}"),
    }
}

pub(super) fn count_label(one: &str, many: &str, count: usize) -> String {
    let count = count.to_string();
    let key = if count == "1" { one } else { many };
    t_with(key, &[("count", &count)])
}

fn import_failure(message: &str, url: &str, expired: bool) -> AnyView {
    if message == t("import.error.blocked_site") {
        return view! {
            <ImportProgress current=1/>
            <h1>{t("import.failure_heading")}</h1>
            <div class="import-warning" role="alert"><p>{message}</p></div>
            <a class="import-primary" href="/recipes/import?source=text">{t("import.source.text")}</a>
            <a class="import-manual" href="/recipes/import?source=photos">{t("import.source.photos")}</a>
        }.into_any();
    }
    let heading = if expired {
        t("import.expired_heading")
    } else {
        t("import.failure_heading")
    };
    let recover = if expired {
        "/recipes/import?source=photos".to_owned()
    } else {
        format!("/recipes/import?source=website&url={}", encode_query(url))
    };
    view! {
        <a class="import-back" href="/recipes/import"><svg aria-hidden="true" viewBox="0 0 24 24"><path d="m15 18-6-6 6-6M9 12h12"/></svg>{t("import.back_review")}</a>
        <ImportProgress current=1/>
        <h1>{heading}</h1>
        <div class="import-warning" role="alert"><svg aria-hidden="true" viewBox="0 0 24 24"><path d="M12 3 2 21h20L12 3z"/><path d="M12 9v4m0 4h.01"/></svg><div><strong>{message}</strong><p>{if expired { t("import.expired_help") } else { t("import.failure_help") }}</p></div></div>
        <a class="import-primary import-retry" href=recover>{t("import.retry")}</a>
        <a class="import-manual" href="/recipes/new">{t("import.manual")}</a>
    }.into_any()
}

fn encode_query(value: &str) -> String {
    value.bytes().fold(String::new(), |mut encoded, byte| {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(hex_digit(byte >> 4));
            encoded.push(hex_digit(byte & 0x0f));
        }
        encoded
    })
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'A' + value - 10),
        _ => '0',
    }
}

#[cfg(test)]
mod tests {
    use super::{encode_query, format_amount};

    #[test]
    fn retry_urls_encode_query_delimiters_and_unicode_bytes() {
        assert_eq!(
            encode_query("https://example.test/?a=one&b=two"),
            "https%3A%2F%2Fexample.test%2F%3Fa%3Done%26b%3Dtwo"
        );
        assert_eq!(encode_query("café"), "caf%C3%A9");
    }

    #[test]
    fn ingredient_amounts_keep_units_and_unspecified_values() {
        assert_eq!(format_amount("2", "piece"), "2");
        assert_eq!(format_amount("2", "tbsp"), "2 tbsp");
        assert_eq!(format_amount("", "to taste"), "to taste");
        assert_eq!(format_amount("", ""), "Unspecified");
    }
}
