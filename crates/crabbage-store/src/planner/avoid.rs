//! What a household avoids, and the recipes that conflict with it.

use crabbage_domain::allergens::Allergen;
use crabbage_domain::warnings::Warning;
use sqlx::QueryBuilder;
use sqlx::sqlite::{Sqlite, SqlitePool};

use super::model::Avoid;
use crate::StoreError;

/// The allergens and warnings household `household` avoids; none when it saved none.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn avoid(pool: &SqlitePool, household: i64) -> Result<Avoid, StoreError> {
    let codes: Vec<String> =
        sqlx::query_scalar("SELECT code FROM household_avoid WHERE household_id = ?")
            .bind(household)
            .fetch_all(pool)
            .await?;
    Ok(Avoid {
        allergens: Allergen::from_codes(codes.iter().map(String::as_str)),
        warnings: Warning::from_codes(codes.iter().map(String::as_str)),
    })
}

/// Replaces what household `household` avoids.
///
/// # Errors
///
/// Returns an error if a query fails; nothing changes then.
pub async fn save_avoid(
    pool: &SqlitePool,
    household: i64,
    avoid: &Avoid,
) -> Result<(), StoreError> {
    let mut tx = crate::begin_write(pool).await?;
    sqlx::query("DELETE FROM household_avoid WHERE household_id = ?")
        .bind(household)
        .execute(&mut *tx)
        .await?;
    for code in avoid.codes() {
        sqlx::query("INSERT INTO household_avoid (household_id, code) VALUES (?, ?)")
            .bind(household)
            .bind(code)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// The ids of the recipes with an avoided allergen in an ingredient or an avoided warning, in id
/// order. None when nothing is avoided.
///
/// # Errors
///
/// Returns an error if the query fails.
pub async fn conflicting_recipes(pool: &SqlitePool, avoid: &Avoid) -> Result<Vec<i64>, StoreError> {
    if avoid.allergens.is_empty() && avoid.warnings.is_empty() {
        return Ok(Vec::new());
    }
    let mut query: QueryBuilder<Sqlite> = QueryBuilder::new(
        "SELECT l.recipe_id FROM recipe_lines l \
         JOIN ingredient_allergens a ON a.ingredient_id = l.ingredient_id WHERE a.allergen IN (",
    );
    push_codes(
        &mut query,
        avoid.allergens.iter().map(|allergen| allergen.code()),
    );
    query.push(") UNION SELECT recipe_id FROM recipe_warnings WHERE warning IN (");
    push_codes(
        &mut query,
        avoid.warnings.iter().map(|warning| warning.code()),
    );
    query.push(") ORDER BY 1");
    Ok(query.build_query_scalar().fetch_all(pool).await?)
}

/// Binds `codes` as a list after an empty code, which matches nothing and keeps the list valid when
/// `codes` is empty.
fn push_codes<'a>(query: &mut QueryBuilder<Sqlite>, codes: impl Iterator<Item = &'a str>) {
    let mut list = query.separated(", ");
    list.push_bind("");
    for code in codes {
        list.push_bind(code.to_owned());
    }
}
