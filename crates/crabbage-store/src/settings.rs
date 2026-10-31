use std::fmt::{self, Debug, Formatter};

use sqlx::sqlite::SqlitePool;

/// Stored AI service configuration. The key is only returned by store code, never by the app API.
#[derive(Clone, PartialEq, Eq)]
pub struct AiSettings {
    /// Whether AI features are enabled.
    pub ai_enabled: bool,
    /// Provider identifier.
    pub provider: String,
    /// `OpenAI` compatible endpoint base URL.
    pub base_url: String,
    /// Model name.
    pub model: String,
    /// Optional API key.
    pub api_key: Option<String>,
}

impl Debug for AiSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("AiSettings")
            .field("ai_enabled", &self.ai_enabled)
            .field("provider", &self.provider)
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("api_key", &self.api_key.as_ref().map(|_| "<redacted>"))
            .finish()
    }
}

impl AiSettings {
    /// Whether AI is enabled with a non-empty base URL and model.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.ai_enabled && !self.base_url.trim().is_empty() && !self.model.trim().is_empty()
    }
}

/// Reads the single AI settings row.
///
/// # Errors
///
/// Returns an error when the database query fails.
pub async fn get(pool: &SqlitePool) -> Result<AiSettings, sqlx::Error> {
    sqlx::query_as::<_, (bool, String, String, String, Option<String>)>(
        "SELECT ai_enabled, provider, base_url, model, api_key FROM ai_settings WHERE id = 1",
    )
    .fetch_one(pool)
    .await
    .map(
        |(ai_enabled, provider, base_url, model, api_key)| AiSettings {
            ai_enabled,
            provider,
            base_url,
            model,
            api_key,
        },
    )
}

/// Saves settings and retains the existing key when `api_key` is absent.
///
/// # Errors
///
/// Returns an error when the database update fails.
pub async fn save(pool: &SqlitePool, settings: &AiSettings) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE ai_settings SET ai_enabled = ?, provider = ?, base_url = ?, model = ?, api_key = COALESCE(?, api_key) WHERE id = 1")
        .bind(settings.ai_enabled)
        .bind(&settings.provider)
        .bind(&settings.base_url)
        .bind(&settings.model)
        .bind(settings.api_key.as_deref())
        .execute(pool)
        .await?;
    Ok(())
}

/// Removes the saved key.
///
/// # Errors
///
/// Returns an error when the database update fails.
pub async fn remove_key(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE ai_settings SET api_key = NULL WHERE id = 1")
        .execute(pool)
        .await?;
    Ok(())
}
