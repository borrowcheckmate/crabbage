//! The photo card of the edit page: the current photo, a form to upload another, and a button to
//! remove the photo when there is one. The forms are plain HTML and post to the photo routes.

use leptos::prelude::*;

use crate::i18n::t;

/// The photo card of recipe `id`, titled `title`, with its photo `image` when it has one.
pub(crate) fn photo_card(id: i64, title: &str, image: Option<&str>) -> AnyView {
    let upload_href = format!("/recipes/{id}/image");
    let remove_href = format!("/recipes/{id}/image/delete");
    let preview = image.map(|name| {
        view! { <img class="photo" src=format!("/images/{name}") alt=title.to_owned()/> }
    });
    let remove = image.is_some().then(|| {
        view! {
            <form method="post" action=remove_href>
                <button class="secondary">{t("recipe.button.remove_photo")}</button>
            </form>
        }
    });
    view! {
        <section class="card">
            <h2>{t("recipe.photo.heading")}</h2>
            {preview}
            <form method="post" enctype="multipart/form-data" action=upload_href>
                <label>{t("recipe.photo.choose")}
                    <input type="file" name="image" accept="image/jpeg,image/png,image/webp"/>
                </label>
                <button>{t("recipe.button.upload")}</button>
            </form>
            {remove}
        </section>
    }
    .into_any()
}
