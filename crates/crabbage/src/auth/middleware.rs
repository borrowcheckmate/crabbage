use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use crabbage_store::sessions;

use super::cookie::SESSION;
use super::{AuthError, SessionUser, session};
use crate::state::AppState;

pub(crate) async fn require_session(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    if is_public(request.uri().path()) {
        return next.run(request).await;
    }
    let Some(token) = SESSION.value(request.headers()) else {
        return Redirect::to("/auth/login").into_response();
    };
    match sessions::find_user(&state.pool, &token, session::now()).await {
        Ok(Some(user_id)) => {
            request.extensions_mut().insert(SessionUser(user_id));
            next.run(request).await
        }
        Ok(None) => Redirect::to("/auth/login").into_response(),
        Err(error) => AuthError::from(error).into_response(),
    }
}

fn is_public(path: &str) -> bool {
    path == "/healthz" || path.starts_with("/auth/") || path.starts_with("/pkg/")
}
