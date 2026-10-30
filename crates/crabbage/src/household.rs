//! Switching the active household of the signed-in user.

use axum::Form;
use axum::Router;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::post;
use crabbage_store::households;
use serde::Deserialize;

use crate::SessionUser;
use crate::state::AppState;

#[derive(Deserialize)]
struct SwitchForm {
    household_id: i64,
}

pub(crate) fn routes() -> Router<AppState> {
    Router::new().route("/household", post(switch))
}

async fn switch(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
    Form(form): Form<SwitchForm>,
) -> Result<Redirect, StatusCode> {
    match households::set_active_household(&state.pool, user.0, form.household_id).await {
        Ok(true) => Ok(Redirect::to("/")),
        Ok(false) => Err(StatusCode::FORBIDDEN),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
