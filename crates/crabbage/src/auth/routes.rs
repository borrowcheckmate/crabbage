use axum::Router;
use axum::routing::{get, post};

use super::{callback, login, logout};
use crate::state::AppState;

/// The signed-in user of a request, set by the session check for every page that needs one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionUser(pub i64);

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", get(login::login))
        .route("/auth/callback", get(callback::callback))
        .route("/auth/logout", post(logout::logout))
}
