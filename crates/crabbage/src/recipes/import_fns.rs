//! Reading a recipe from a web page and saving the reviewed result. Any signed-in user may import.

#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces each server function body with a stub that does not await"
    )
)]

use leptos::prelude::ServerFnError;
use leptos::server;

use super::types::ImportPreview;

/// Reads the recipe at `url` and completes it against the catalogue. Page and fetch failures come
/// back as translated messages.
#[server(prefix = "/api", endpoint = "import_recipe")]
pub(crate) async fn import_recipe(url: String) -> Result<ImportPreview, ServerFnError> {
    let (pool, _user) = crate::access::signed_in().await?;
    let config = crate::access::config()?;
    crate::recipes::import::read_page(&pool, &config, &url).await
}

/// Whether the AI is configured, so the page offers reading photos.
#[server(prefix = "/api", endpoint = "ai_enabled")]
pub(crate) async fn ai_enabled() -> Result<bool, ServerFnError> {
    crate::access::signed_in().await?;
    Ok(crate::access::config()?.ai_enabled())
}

/// The preview of the photo import stored under `token`, only for the user who made it. An unknown,
/// expired or foreign token comes back as the translated expiry message.
#[server(prefix = "/api", endpoint = "photo_preview")]
pub(crate) async fn photo_preview(token: String) -> Result<ImportPreview, ServerFnError> {
    use crate::recipes::photo_drafts::PhotoDrafts;
    let (_pool, user) = crate::access::signed_in().await?;
    leptos::prelude::use_context::<std::sync::Arc<PhotoDrafts>>()
        .and_then(|drafts| drafts.get(&token, user, std::time::Instant::now()))
        .ok_or_else(|| ServerFnError::new(crate::i18n::t("import.photos_expired")))
}

/// Creates a recipe from the reviewed JSON draft like `create_recipe` does, then downloads the
/// photo at `image`, if any, and attaches it. A photo that cannot be downloaded or stored does not
/// fail the save.
#[server(prefix = "/api", endpoint = "save_import")]
pub(crate) async fn save_import(
    draft: String,
    image: Option<String>,
    photo_token: Option<String>,
    keep_photo: Option<bool>,
) -> Result<i64, ServerFnError> {
    use crabbage_store::recipes::create;
    let (pool, user) = crate::access::signed_in().await?;
    let config = crate::access::config()?;
    let input = crate::recipes::draft::parse(&draft)?;
    let id = create(&pool, user, &input, crate::auth::now())
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    if let Some(url) = image
        .as_deref()
        .map(str::trim)
        .filter(|url| !url.is_empty())
    {
        crate::recipes::import_photo::attach(&pool, &config, id, url).await;
    }
    if keep_photo == Some(true) {
        use crate::images::{Kind, write};
        use crabbage_store::recipes::set_image;
        if let (Some(token), Some(drafts)) = (
            photo_token.as_deref(),
            leptos::prelude::use_context::<std::sync::Arc<crate::recipes::PhotoDrafts>>(),
        ) && let Some(bytes) = drafts.photo(token, user, std::time::Instant::now())
            && let Ok(name) = write(&config.data_dir, &bytes, Kind::Jpeg).await
            && set_image(&pool, id, Some(&name)).await.is_err()
        {
            drop(crate::images::remove(&config.data_dir, &name).await);
        }
    }
    Ok(id)
}
