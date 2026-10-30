//! Server-side login sessions, looked up by a hash of the cookie token.

use sha2::{Digest, Sha256};
use sqlx::sqlite::SqlitePool;

use crate::StoreError;

/// Stores a session for `user_id` that is valid until `expires_at` (unix seconds).
///
/// Only the SHA-256 hash of `token` is stored.
///
/// # Errors
///
/// Returns an error if the query fails, including when `user_id` does not exist.
pub async fn create(
    pool: &SqlitePool,
    token: &str,
    user_id: i64,
    expires_at: i64,
) -> Result<(), StoreError> {
    sqlx::query("INSERT INTO sessions (id_hash, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(hash(token))
        .bind(user_id)
        .bind(expires_at)
        .execute(pool)
        .await?;
    Ok(())
}

/// Returns the user id of the session for `token` if it has not expired at `now` (unix seconds).
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn find_user(
    pool: &SqlitePool,
    token: &str,
    now: i64,
) -> Result<Option<i64>, StoreError> {
    let user_id =
        sqlx::query_scalar("SELECT user_id FROM sessions WHERE id_hash = ? AND expires_at > ?")
            .bind(hash(token))
            .bind(now)
            .fetch_optional(pool)
            .await?;
    Ok(user_id)
}

/// Deletes the session for `token`. Deleting a missing session is not an error.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn delete(pool: &SqlitePool, token: &str) -> Result<(), StoreError> {
    sqlx::query("DELETE FROM sessions WHERE id_hash = ?")
        .bind(hash(token))
        .execute(pool)
        .await?;
    Ok(())
}

fn hash(token: &str) -> String {
    let hex: Vec<String> = Sha256::digest(token.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    hex.concat()
}
