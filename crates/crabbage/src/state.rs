//! The state shared by every request handler.

use std::sync::Arc;

use axum::extract::FromRef;
use leptos::config::LeptosOptions;
use openidconnect::reqwest;
use sqlx::sqlite::SqlitePool;

use crate::auth::Oidc;
use crate::i18n::Language;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) options: LeptosOptions,
    pub(crate) pool: SqlitePool,
    pub(crate) http: reqwest::Client,
    pub(crate) oidc: Arc<Oidc>,
    pub(crate) admin_group: String,
    pub(crate) household_group_prefix: String,
    pub(crate) language: Language,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(input: &AppState) -> Self {
        input.options.clone()
    }
}
