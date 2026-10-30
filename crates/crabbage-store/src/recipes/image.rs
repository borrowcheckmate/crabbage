//! The photo of a recipe: the file name of its image in the images directory.

use sqlx::sqlite::SqlitePool;

use crate::StoreError;

/// Sets the image of recipe `id` to `image`, or removes it when `None`, and returns the file name it
/// replaced. The result is `None` when there was no image or no such recipe.
///
/// # Errors
///
/// Returns an error if a query fails.
pub async fn set_image(
    pool: &SqlitePool,
    id: i64,
    image: Option<&str>,
) -> Result<Option<String>, StoreError> {
    let mut tx = crate::begin_write(pool).await?;
    let previous: Option<String> =
        sqlx::query_scalar::<_, Option<String>>("SELECT image FROM recipes WHERE id = ?")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
            .flatten();
    sqlx::query("UPDATE recipes SET image = ? WHERE id = ?")
        .bind(image)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(previous)
}
