use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::response::{IntoResponse, Redirect};
use crabbage_store::login_attempts;

use super::AuthError;
use super::cookie::LOGIN;
use super::session;
use crate::state::AppState;

pub(super) async fn login(State(state): State<AppState>) -> Result<impl IntoResponse, AuthError> {
    let (url, state_value, attempt) = state.oidc.authorization();
    login_attempts::create(&state.pool, &state_value, &attempt, session::now()).await?;
    Ok((
        [(SET_COOKIE, LOGIN.set(&state_value))],
        Redirect::to(url.as_str()),
    ))
}
