//! The state shared by every request handler.

use std::sync::Arc;

use axum::extract::FromRef;
use leptos::config::LeptosOptions;
use openidconnect::reqwest;
use sqlx::sqlite::SqlitePool;
use tokio::sync::OnceCell;

use crate::auth::{AuthError, Oidc};
use crate::config::Config;
use crate::i18n::Language;
use crate::recipes::PhotoDrafts;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) options: LeptosOptions,
    pub(crate) pool: SqlitePool,
    pub(crate) http: reqwest::Client,
    pub(crate) config: Arc<Config>,
    pub(crate) oidc: Arc<OnceCell<Oidc>>,
    pub(crate) admin_group: String,
    pub(crate) household_group_prefix: String,
    pub(crate) language: Language,
    pub(crate) photo_drafts: Arc<PhotoDrafts>,
}

impl AppState {
    /// The `OpenID` Connect client. The provider is discovered on first use and the client is kept
    /// only once discovery succeeds, so a provider that is down is tried again on the next request.
    pub(crate) async fn oidc(&self) -> Result<&Oidc, AuthError> {
        Ok(self
            .oidc
            .get_or_try_init(|| Oidc::discover(&self.config, &self.http))
            .await?)
    }
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(input: &AppState) -> Self {
        input.options.clone()
    }
}
