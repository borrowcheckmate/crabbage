use super::{Draft, ImportError, Limits, extract, fetch};

/// Fetches the page at `url` under `limits` and reads its recipe.
///
/// # Errors
///
/// Any [`ImportError`] from fetching the page or from finding its recipe.
pub async fn import(url: &str, limits: &Limits) -> Result<Draft, ImportError> {
    let html = fetch(url, limits).await?;
    extract(&html)
}
