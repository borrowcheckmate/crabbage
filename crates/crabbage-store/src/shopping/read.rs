//! Reading a household's shopping list.

use sqlx::sqlite::SqlitePool;

use super::model::ShoppingItem;
use crate::StoreError;

/// The items of `household`: unchecked items first, then checked ones, each in the order they
/// were added.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn list(pool: &SqlitePool, household: i64) -> Result<Vec<ShoppingItem>, StoreError> {
    let items = sqlx::query_as(
        "SELECT s.id, s.ingredient_id, i.name AS ingredient, s.text, s.amount, s.unit, s.checked, \
         s.recipe_id, s.plan_entry_id, i.store_section_id AS section_id, sec.position AS section_position \
         FROM shopping_items s \
         LEFT JOIN ingredients i ON i.id = s.ingredient_id \
         LEFT JOIN store_sections sec ON sec.id = i.store_section_id \
         WHERE s.household_id = ? ORDER BY s.checked, s.id",
    )
    .bind(household)
    .fetch_all(pool)
    .await?;
    Ok(items)
}
