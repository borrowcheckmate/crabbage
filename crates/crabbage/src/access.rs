//! Who may use the app: the signed-in user, and the admins who may manage the catalog.

use axum::Extension;
use axum::http::StatusCode;
use crabbage_domain::membership::Role;
use crabbage_domain::permissions::can_manage_catalog;
use crabbage_store::{StoreError, users};
use leptos::prelude::{ServerFnError, use_context};
use leptos_axum::{ResponseOptions, extract};
use sqlx::sqlite::SqlitePool;

use crate::SessionUser;
use crate::i18n::t;

/// The pool and the id of the signed-in user.
///
/// # Errors
///
/// Returns an error when the request has no session user or no pool is provided.
pub(crate) async fn signed_in() -> Result<(SqlitePool, i64), ServerFnError> {
    let Extension(user) = extract::<Extension<SessionUser>>().await?;
    let pool = use_context::<SqlitePool>()
        .ok_or_else(|| ServerFnError::new(t("catalog.error.database")))?;
    Ok((pool, user.0))
}

/// The pool, once the signed-in user is known to be an admin. Otherwise the response is 403 and the
/// error says the user is not allowed to manage the catalog.
///
/// # Errors
///
/// Returns an error for non-admins and when the store fails.
pub(crate) async fn require_admin() -> Result<SqlitePool, ServerFnError> {
    let (pool, user) = signed_in().await?;
    let is_admin = users::find(&pool, user)
        .await
        .map_err(|error| store_failure(&error))?
        .is_some_and(|found| found.is_admin);
    let role = if is_admin { Role::Admin } else { Role::Member };
    if can_manage_catalog(role) {
        return Ok(pool);
    }
    if let Some(options) = use_context::<ResponseOptions>() {
        options.set_status(StatusCode::FORBIDDEN);
    }
    Err(ServerFnError::new(t("admin.not_allowed")))
}

/// The text a failed store call shows to the user.
pub(crate) fn store_failure(error: &StoreError) -> ServerFnError {
    match error {
        StoreError::Conflict(_) => ServerFnError::new(t("catalog.error.taken")),
        StoreError::InUse => ServerFnError::new(t("catalog.error.in_use")),
        StoreError::Io(_) | StoreError::Database(_) | StoreError::Migrate(_) => {
            ServerFnError::new(t("catalog.error.database"))
        }
    }
}

/// The trimmed text, or an error when it is empty.
pub(crate) fn clean(value: &str) -> Result<&str, ServerFnError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ServerFnError::new(t("catalog.error.empty")));
    }
    Ok(trimmed)
}

/// Succeeds when the store found the row it changed, and says it is missing otherwise.
pub(crate) fn ensure_found(found: bool) -> Result<(), ServerFnError> {
    if found {
        return Ok(());
    }
    Err(ServerFnError::new(t("catalog.error.missing")))
}
