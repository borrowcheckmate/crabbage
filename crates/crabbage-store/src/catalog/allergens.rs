//! The allergens of catalogue ingredients.

use crabbage_domain::allergens::Allergen;
use sqlx::sqlite::SqlitePool;

use crate::StoreError;

/// Replaces the allergens of the ingredient `id`. Returns `false`, and changes nothing, when
/// there is no such ingredient.
///
/// # Errors
///
/// Returns an error if a query fails. Nothing is changed in that case.
pub async fn set_allergens(
    pool: &SqlitePool,
    id: i64,
    allergens: &[Allergen],
) -> Result<bool, StoreError> {
    let mut tx = crate::begin_write(pool).await?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM ingredients WHERE id = ?)")
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    if !exists {
        return Ok(false);
    }
    sqlx::query("DELETE FROM ingredient_allergens WHERE ingredient_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for allergen in allergens {
        sqlx::query(
            "INSERT OR IGNORE INTO ingredient_allergens (ingredient_id, allergen) VALUES (?, ?)",
        )
        .bind(id)
        .bind(allergen.code())
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(true)
}

pub(crate) async fn all(pool: &SqlitePool) -> Result<Vec<(i64, Allergen)>, StoreError> {
    let rows: Vec<(i64, String)> =
        sqlx::query_as("SELECT ingredient_id, allergen FROM ingredient_allergens")
            .fetch_all(pool)
            .await?;
    Ok(rows
        .into_iter()
        .filter_map(|(id, code)| Allergen::from_code(&code).map(|allergen| (id, allergen)))
        .collect())
}
