//! Server functions over HTTP. Every `#[server]` function is served under `/api`, behind the
//! session check.

use std::sync::Arc;

use axum::Router;
use axum::extract::{Request, State};
use axum::response::IntoResponse;
use axum::routing::post;
use leptos::prelude::provide_context;
use leptos_axum::handle_server_fns_with_context;

use crate::state::AppState;

pub(crate) fn routes() -> Router<AppState> {
    Router::new().route("/api/{*fn_name}", post(serve))
}

async fn serve(State(state): State<AppState>, request: Request) -> impl IntoResponse {
    let pool = state.pool.clone();
    let language = state.language;
    let config = Arc::clone(&state.config);
    handle_server_fns_with_context(
        move || {
            provide_context(pool.clone());
            provide_context(language);
            provide_context(Arc::clone(&config));
        },
        request,
    )
    .await
}
