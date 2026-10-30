//! Who may use the app: the signed-in user, their role, and the refusals for what they may not do.

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

/// The role of user `user`. A user the store does not know is a member.
///
/// # Errors
///
/// Returns an error when the store fails.
pub(crate) async fn role(pool: &SqlitePool, user: i64) -> Result<Role, ServerFnError> {
    let is_admin = users::find(pool, user)
        .await
        .map_err(|error| store_failure(&error))?
        .is_some_and(|found| found.is_admin);
    Ok(if is_admin { Role::Admin } else { Role::Member })
}

/// The pool, once the signed-in user is known to be an admin. Otherwise the response is 403 and the
/// error says the user is not allowed to manage the catalog.
///
/// # Errors
///
/// Returns an error for non-admins and when the store fails.
pub(crate) async fn require_admin() -> Result<SqlitePool, ServerFnError> {
    let (pool, user) = signed_in().await?;
    if can_manage_catalog(role(&pool, user).await?) {
        return Ok(pool);
    }
    Err(refuse(t("admin.not_allowed")))
}

/// The error for a refused request: the response is 403 and the error carries `message`.
pub(crate) fn refuse(message: &str) -> ServerFnError {
    if let Some(options) = use_context::<ResponseOptions>() {
        options.set_status(StatusCode::FORBIDDEN);
    }
    ServerFnError::new(message)
}

/// The text a failed store call shows to the user.
pub(crate) fn store_failure(error: &StoreError) -> ServerFnError {
    match error {
        StoreError::Conflict(_) => ServerFnError::new(t("catalog.error.taken")),
        StoreError::InUse => ServerFnError::new(t("catalog.error.in_use")),
        StoreError::UnknownLine => ServerFnError::new(t("recipe.error.line")),
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
