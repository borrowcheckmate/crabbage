//! Rules shared by ingredients and store sections: a name is unique regardless of case, and a name
//! never equals an alias, nor an alias a name.

use sqlx::SqliteConnection;
use sqlx::error::DatabaseError;

use crate::StoreError;

/// Maps a failed write to a conflict on `text` when it breaks a unique constraint, and any other
/// failure to a database error.
pub(super) fn unique_or_database(text: &str) -> impl FnOnce(sqlx::Error) -> StoreError + '_ {
    move |error| {
        if error
            .as_database_error()
            .is_some_and(DatabaseError::is_unique_violation)
        {
            StoreError::Conflict(text.to_owned())
        } else {
            StoreError::Database(error)
        }
    }
}

/// Fails with a conflict when `name` is already an alias.
pub(super) async fn ensure_not_an_alias(
    conn: &mut SqliteConnection,
    name: &str,
) -> Result<(), StoreError> {
    let taken: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM ingredient_aliases WHERE alias = ?)")
            .bind(name)
            .fetch_one(conn)
            .await?;
    if taken {
        return Err(StoreError::Conflict(name.to_owned()));
    }
    Ok(())
}

/// Fails with a conflict when `alias` is already an ingredient name.
pub(super) async fn ensure_not_a_name(
    conn: &mut SqliteConnection,
    alias: &str,
) -> Result<(), StoreError> {
    let taken: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM ingredients WHERE name = ?)")
            .bind(alias)
            .fetch_one(conn)
            .await?;
    if taken {
        return Err(StoreError::Conflict(alias.to_owned()));
    }
    Ok(())
}
