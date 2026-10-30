//! `GET /images/:name`. The session check runs before this, so only signed-in users reach it.

use axum::Router;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;

use super::files::read;
use super::naming::kind_of;
use crate::state::AppState;

const CACHE_CONTROL: &str = "private, max-age=31536000, immutable";

pub(crate) fn routes() -> Router<AppState> {
    Router::new().route("/images/{name}", get(serve))
}

async fn serve(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Response, StatusCode> {
    let kind = kind_of(&name).ok_or(StatusCode::NOT_FOUND)?;
    let bytes = read(&state.config.data_dir, &name).await.map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    Ok((
        [
            (header::CONTENT_TYPE, kind.content_type()),
            (header::CACHE_CONTROL, CACHE_CONTROL),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        bytes,
    )
        .into_response())
}
