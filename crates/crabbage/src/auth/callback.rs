use axum::extract::{Query, State};
use axum::http::{HeaderMap, header::SET_COOKIE};
use axum::response::{AppendHeaders, IntoResponse, Redirect};
use crabbage_domain::membership::{Membership, Role};
use crabbage_store::{households, login_attempts, sessions, users};
use serde::Deserialize;

use super::cookie::{LOGIN, SESSION};
use super::{AuthError, session};
use crate::state::AppState;

#[derive(Deserialize)]
pub(super) struct CallbackQuery {
    code: String,
    state: String,
}

pub(super) async fn callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CallbackQuery>,
) -> Result<impl IntoResponse, AuthError> {
    if LOGIN.value(&headers).as_deref() != Some(query.state.as_str()) {
        return Err(AuthError::Rejected("login does not match this browser"));
    }
    let oidc = state.oidc().await?;
    let now = session::now();
    let attempt = login_attempts::take(&state.pool, &query.state, now)
        .await?
        .ok_or(AuthError::Rejected("unknown or expired login"))?;
    let identity = oidc
        .identity(&state.http, query.code, attempt)
        .await
        .ok_or(AuthError::Rejected("login failed"))?;
    let user_id = users::upsert_user(&state.pool, &identity.subject, &identity.name).await?;
    let membership = Membership::from_groups(
        &identity.groups,
        &state.admin_group,
        &state.household_group_prefix,
    );
    households::sync_membership(
        &state.pool,
        user_id,
        membership.role == Role::Admin,
        &membership.households,
    )
    .await?;
    let token = session::new_token()?;
    sessions::create(&state.pool, &token, user_id, now + session::LIFETIME_SECS).await?;
    Ok((
        AppendHeaders([
            (SET_COOKIE, SESSION.set(&token)),
            (SET_COOKIE, LOGIN.clear()),
        ]),
        Redirect::to("/"),
    ))
}
