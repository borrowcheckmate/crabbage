//! Reading a recipe from photos. A plain HTML form posts up to four images here; the AI reads them
//! and the result waits in [`PhotoDrafts`] for the import page to show.
//!
//! [`PhotoDrafts`]: super::photo_drafts::PhotoDrafts

use std::time::Instant;

use axum::Extension;
use axum::Router;
use axum::extract::{DefaultBodyLimit, Form, Multipart, State};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::post;
use crabbage_import::ai::{AiClient, Image};
use crabbage_import::pipeline::from_photos;
use crabbage_store::catalog::ingredients::list;
use crabbage_store::settings;
use serde::Deserialize;

use super::import::{candidates, preview};
use super::photo::MAX_UPLOAD_BYTES;
use crate::SessionUser;
use crate::access::internal_error;
use crate::i18n::t_with;
use crate::images::Kind;
use crate::state::AppState;

const MAX_PHOTOS: usize = 4;
const FRAMING_BYTES: usize = 64 * 1024;
const MAX_TEXT_BYTES: usize = 64 * 1024;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/recipes/import/photos",
            post(upload).layer(DefaultBodyLimit::max(
                MAX_PHOTOS * MAX_UPLOAD_BYTES + FRAMING_BYTES,
            )),
        )
        .route(
            "/recipes/import/text",
            post(upload_text).layer(DefaultBodyLimit::max(MAX_TEXT_BYTES)),
        )
}

#[derive(Deserialize)]
struct TextForm {
    text: String,
    source_url: Option<String>,
}

async fn upload_text(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
    Form(form): Form<TextForm>,
) -> Result<Redirect, StatusCode> {
    if form.text.trim().is_empty() {
        return Ok(back("source=text&photo_error=text_invalid"));
    }
    let source = form.source_url.as_deref().unwrap_or_default().trim();
    let preview = super::import::read_text(&state.pool, &state.config, &form.text, source)
        .await
        .map_err(|_error| StatusCode::INTERNAL_SERVER_ERROR)?;
    let token = state
        .photo_drafts
        .insert(user.0, preview, None, Instant::now())
        .map_err(|_error| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut query = url::form_urlencoded::Serializer::new(format!("text={token}"));
    if !source.is_empty() {
        query.append_pair("source_url", source);
    }
    Ok(back(&query.finish()))
}

async fn upload(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
    mut multipart: Multipart,
) -> Result<Redirect, StatusCode> {
    let ai = settings::get(&state.pool).await.map_err(internal_error)?;
    if !ai.is_enabled() {
        return Err(StatusCode::NOT_FOUND);
    }
    let base_url = url::Url::parse(&ai.base_url).map_err(|_error| StatusCode::NOT_FOUND)?;
    if !matches!(base_url.scheme(), "http" | "https") {
        return Err(StatusCode::NOT_FOUND);
    }
    let images = images(&mut multipart).await?;
    if images.is_empty() {
        return Ok(back("photo_error=invalid"));
    }
    let count = images.len().to_string();
    let ingredients = list(&state.pool).await.map_err(internal_error)?;
    let client = AiClient::new(base_url, ai.api_key, ai.model);
    let language = state.config.language.code();
    let Ok(imported) =
        from_photos(images.clone(), &candidates(&ingredients), &client, language).await
    else {
        return Ok(back("photo_error=ai"));
    };
    let photo = match imported.photo {
        Some(bounds) => tokio::task::spawn_blocking(move || crate::images::crop(&images, bounds))
            .await
            .ok()
            .flatten(),
        None => None,
    };
    let source = t_with("import.photos_count", &[("count", &count)]);
    let now = Instant::now();
    let token = state
        .photo_drafts
        .insert(user.0, preview(&imported, &source, true), photo, now)
        .map_err(internal_error)?;
    Ok(back(&format!("photos={token}")))
}

/// The first four `photo` fields that are within the size limit and a JPEG, PNG or WebP.
async fn images(multipart: &mut Multipart) -> Result<Vec<Image>, StatusCode> {
    let mut images = Vec::new();
    while images.len() < MAX_PHOTOS {
        let Some(field) = multipart
            .next_field()
            .await
            .map_err(|error| error.status())?
        else {
            break;
        };
        if field.name() != Some("photo") {
            continue;
        }
        let bytes = field.bytes().await.map_err(|error| error.status())?;
        if bytes.len() > MAX_UPLOAD_BYTES {
            continue;
        }
        if let Some(kind) = Kind::detect(&bytes) {
            images.push(Image {
                media_type: kind.content_type().to_owned(),
                bytes: bytes.to_vec(),
            });
        }
    }
    Ok(images)
}

fn back(query: &str) -> Redirect {
    Redirect::to(&format!("/recipes/import?{query}"))
}
