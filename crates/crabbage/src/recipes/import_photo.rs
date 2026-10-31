//! Downloads the photo of an imported recipe and stores it like an uploaded one.

use crabbage_import::web::{Limits, fetch_bytes};
use crabbage_store::recipes::set_image;
use sqlx::sqlite::SqlitePool;

use super::photo::MAX_UPLOAD_BYTES;
use crate::config::Config;
use crate::images::{Kind, remove, write};

/// Downloads `url` through the safe fetch, checks that it is a JPEG, PNG or WebP within the size an
/// upload may have, and makes it the photo of recipe `id`. Returns whether the recipe has the photo
/// now; any failure leaves the recipe as it is.
pub(crate) async fn attach(pool: &SqlitePool, config: &Config, id: i64, url: &str) -> bool {
    let limits = Limits {
        max_bytes: MAX_UPLOAD_BYTES,
        allow_private: config.import_allow_private,
        ..Limits::default()
    };
    let Ok(fetched) = fetch_bytes(url, &limits).await else {
        return false;
    };
    let Some(kind) = Kind::detect(&fetched.bytes) else {
        return false;
    };
    let Ok(name) = write(&config.data_dir, &fetched.bytes, kind).await else {
        return false;
    };
    if set_image(pool, id, Some(&name)).await.is_ok() {
        return true;
    }
    drop(remove(&config.data_dir, &name).await);
    false
}
