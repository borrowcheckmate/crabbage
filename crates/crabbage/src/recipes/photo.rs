//! Uploading and removing a recipe's photo. Plain HTML forms post to these routes, so they answer
//! with a redirect to the edit page. Only the users who may change the recipe may use them.

use axum::Extension;
use axum::Router;
use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::post;
use crabbage_store::recipes::{get, set_image};

use super::rights::Rights;
use crate::SessionUser;
use crate::access::{internal_error, role};
use crate::images::{Kind, remove, write};
use crate::state::AppState;

/// The largest upload accepted: the request body, which holds the image and its multipart framing.
const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/recipes/{id}/image",
            post(upload).layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES)),
        )
        .route("/recipes/{id}/image/delete", post(delete_photo))
}

async fn upload(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
    Path(id): Path<i64>,
    mut multipart: Multipart,
) -> Result<Redirect, StatusCode> {
    may_edit(&state, user.0, id).await?;
    let bytes = image_bytes(&mut multipart).await?;
    let kind = Kind::detect(&bytes).ok_or(StatusCode::UNSUPPORTED_MEDIA_TYPE)?;
    let data_dir = &state.config.data_dir;
    let name = write(data_dir, &bytes, kind)
        .await
        .map_err(internal_error)?;
    let previous = match set_image(&state.pool, id, Some(&name)).await {
        Ok(previous) => previous,
        Err(error) => {
            drop(remove(data_dir, &name).await);
            return Err(internal_error(error));
        }
    };
    if let Some(previous) = previous {
        drop(remove(data_dir, &previous).await);
    }
    Ok(edit_page(id))
}

async fn delete_photo(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
    Path(id): Path<i64>,
) -> Result<Redirect, StatusCode> {
    may_edit(&state, user.0, id).await?;
    let previous = set_image(&state.pool, id, None)
        .await
        .map_err(internal_error)?;
    if let Some(previous) = previous {
        drop(remove(&state.config.data_dir, &previous).await);
    }
    Ok(edit_page(id))
}

/// The first multipart field named `image`, read in full.
async fn image_bytes(multipart: &mut Multipart) -> Result<Vec<u8>, StatusCode> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| error.status())?
    {
        if field.name() == Some("image") {
            let bytes = field.bytes().await.map_err(|error| error.status())?;
            return Ok(bytes.to_vec());
        }
    }
    Err(StatusCode::BAD_REQUEST)
}

/// Checks that user `user` may change recipe `id`: 404 for a missing recipe, 403 for anyone the
/// recipe's rule does not allow.
async fn may_edit(state: &AppState, user: i64, id: i64) -> Result<(), StatusCode> {
    let recipe = get(&state.pool, id)
        .await
        .map_err(internal_error)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let role = role(&state.pool, user).await.map_err(internal_error)?;
    if Rights::of(role, user, &recipe).edit {
        return Ok(());
    }
    Err(StatusCode::FORBIDDEN)
}

fn edit_page(id: i64) -> Redirect {
    Redirect::to(&format!("/recipes/{id}/edit"))
}
