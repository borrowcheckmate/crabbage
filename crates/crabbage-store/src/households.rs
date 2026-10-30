//! Households, their members, and each user's active household.

use sqlx::sqlite::SqlitePool;

use crate::StoreError;

/// A household: the identity provider group that names it.
#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Household {
    /// The household's id.
    pub id: i64,
    /// The full name of the identity provider group.
    pub group_name: String,
}

/// Stores a login's admin flag and households in one transaction.
///
/// Each group becomes a household on first sight. The user's memberships are replaced by exactly
/// `household_groups`. If the active household is no longer a membership, or none is set, the
/// first membership by group name becomes active.
///
/// # Errors
///
/// Returns an error if a query fails. Nothing is changed in that case.
pub async fn sync_membership(
    pool: &SqlitePool,
    user_id: i64,
    is_admin: bool,
    household_groups: &[String],
) -> Result<(), StoreError> {
    let mut tx = crate::begin_write(pool).await?;
    sqlx::query("UPDATE users SET is_admin = ? WHERE id = ?")
        .bind(is_admin)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM user_households WHERE user_id = ?")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    for group in household_groups {
        let household_id: i64 = sqlx::query_scalar(
            "INSERT INTO households (group_name) VALUES (?) \
             ON CONFLICT (group_name) DO UPDATE SET group_name = excluded.group_name \
             RETURNING id",
        )
        .bind(group)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query(
            "INSERT INTO user_households (user_id, household_id) VALUES (?, ?) \
             ON CONFLICT DO NOTHING",
        )
        .bind(user_id)
        .bind(household_id)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query(
        "UPDATE users SET active_household_id = NULL \
         WHERE id = ? AND active_household_id NOT IN \
         (SELECT household_id FROM user_households WHERE user_id = ?)",
    )
    .bind(user_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE users SET active_household_id = ( \
             SELECT uh.household_id FROM user_households uh \
             JOIN households h ON h.id = uh.household_id \
             WHERE uh.user_id = users.id ORDER BY h.group_name LIMIT 1) \
         WHERE id = ? AND active_household_id IS NULL",
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// The households of a user, ordered by group name.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn households_of(pool: &SqlitePool, user_id: i64) -> Result<Vec<Household>, StoreError> {
    let households = sqlx::query_as(
        "SELECT h.id, h.group_name FROM households h \
         JOIN user_households uh ON uh.household_id = h.id \
         WHERE uh.user_id = ? ORDER BY h.group_name",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(households)
}

/// The active household of a user, if they have one.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn active_household(pool: &SqlitePool, user_id: i64) -> Result<Option<i64>, StoreError> {
    let active: Option<i64> =
        sqlx::query_scalar("SELECT active_household_id FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    Ok(active)
}

/// Makes `household_id` the active household of a user who is a member of it.
///
/// Returns `false`, and changes nothing, when the user is not a member.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn set_active_household(
    pool: &SqlitePool,
    user_id: i64,
    household_id: i64,
) -> Result<bool, StoreError> {
    let result = sqlx::query(
        "UPDATE users SET active_household_id = ? WHERE id = ? AND EXISTS \
         (SELECT 1 FROM user_households WHERE user_id = ? AND household_id = ?)",
    )
    .bind(household_id)
    .bind(user_id)
    .bind(user_id)
    .bind(household_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
