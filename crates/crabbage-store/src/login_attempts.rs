//! Pending logins, from the redirect to the provider until the callback uses them.

use sqlx::sqlite::SqlitePool;

use crate::StoreError;

const LIFETIME_SECS: i64 = 600;

/// The values kept for a login that has started and not yet returned from the provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginAttempt {
    /// The nonce sent with the authorization request.
    pub nonce: String,
    /// The PKCE code verifier matching the challenge sent with the authorization request.
    pub pkce_verifier: String,
}

/// Records a pending login under `state` and removes the attempts that have expired.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn create(
    pool: &SqlitePool,
    state: &str,
    attempt: &LoginAttempt,
    now: i64,
) -> Result<(), StoreError> {
    sqlx::query("DELETE FROM login_attempts WHERE created_at <= ?")
        .bind(now - LIFETIME_SECS)
        .execute(pool)
        .await?;
    sqlx::query(
        "INSERT INTO login_attempts (state, nonce, pkce_verifier, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(state)
    .bind(&attempt.nonce)
    .bind(&attempt.pkce_verifier)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

/// Removes the pending login under `state` and returns it if it has not expired at `now`.
///
/// The attempt is removed even when it has expired, so each state is usable once.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn take(
    pool: &SqlitePool,
    state: &str,
    now: i64,
) -> Result<Option<LoginAttempt>, StoreError> {
    let row: Option<(String, String, i64)> = sqlx::query_as(
        "DELETE FROM login_attempts WHERE state = ? RETURNING nonce, pkce_verifier, created_at",
    )
    .bind(state)
    .fetch_optional(pool)
    .await?;
    Ok(row
        .filter(|(_, _, created_at)| *created_at > now - LIFETIME_SECS)
        .map(|(nonce, pkce_verifier, _)| LoginAttempt {
            nonce,
            pkce_verifier,
        }))
}
