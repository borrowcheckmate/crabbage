use axum::extract::State;
use axum::http::{HeaderMap, header::SET_COOKIE};
use axum::response::{IntoResponse, Redirect};
use crabbage_store::sessions;

use super::AuthError;
use super::cookie::SESSION;
use crate::state::AppState;

pub(super) async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AuthError> {
    if let Some(token) = SESSION.value(&headers) {
        sessions::delete(&state.pool, &token).await?;
    }
    Ok(([(SET_COOKIE, SESSION.clear())], Redirect::to("/")))
}
