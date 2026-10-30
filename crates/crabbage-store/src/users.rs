//! Users, identified by the `OpenID` Connect subject.

use sqlx::sqlite::SqlitePool;

use crate::StoreError;

/// Creates the user for an `OpenID` Connect `subject`, or updates the name of the existing one.
///
/// Returns the user's id.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn upsert_user(pool: &SqlitePool, subject: &str, name: &str) -> Result<i64, StoreError> {
    let id = sqlx::query_scalar(
        "INSERT INTO users (subject, name) VALUES (?, ?) \
         ON CONFLICT (subject) DO UPDATE SET name = excluded.name \
         RETURNING id",
    )
    .bind(subject)
    .bind(name)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

/// A user's name and whether they are an admin.
#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct User {
    /// The name from the latest login.
    pub name: String,
    /// Whether the last login carried the admin group.
    pub is_admin: bool,
}

/// The user with `user_id`, or `None` when there is none.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn find(pool: &SqlitePool, user_id: i64) -> Result<Option<User>, StoreError> {
    let user = sqlx::query_as("SELECT name, is_admin FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}
