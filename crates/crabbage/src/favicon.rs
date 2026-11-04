use axum::Router;
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::get;

use crate::state::AppState;

/// The address of the app's icon. It needs no session, so the sign-in pages show it too.
pub(crate) const FAVICON_PATH: &str = "/favicon.svg";

const FAVICON: &str = include_str!("../assets/favicon.svg");

pub(crate) fn routes() -> Router<AppState> {
    Router::new().route(FAVICON_PATH, get(favicon))
}

async fn favicon() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "image/svg+xml"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        FAVICON,
    )
}
