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
