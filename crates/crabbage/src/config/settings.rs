use std::fmt::{self, Debug, Formatter};
use std::path::PathBuf;

use url::Url;

use super::ConfigError;
use super::vars::Vars;
use crate::i18n::Language;

const REDACTED: &str = "<redacted>";

/// The AI service settings. Present only when AI features are enabled.
#[derive(Clone)]
pub struct AiConfig {
    /// Base URL of the OpenAI-compatible API.
    pub base_url: Url,
    /// API key sent to the service, if it requires one.
    pub api_key: Option<String>,
    /// Name of the model to request.
    pub model: String,
}

impl Debug for AiConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("AiConfig")
            .field("base_url", &self.base_url)
            .field("api_key", &self.api_key.as_ref().map(|_| REDACTED))
            .field("model", &self.model)
            .finish()
    }
}

/// The server settings, read from the environment at startup.
#[derive(Clone)]
pub struct Config {
    /// Language of the app's text, from `CRABBAGE_LANGUAGE` (default `en`).
    pub language: Language,
    /// Directory for persistent data, from `CRABBAGE_DATA_DIR` (default `/data`).
    pub data_dir: PathBuf,
    /// Public URL the app is served at, from `CRABBAGE_PUBLIC_URL`.
    pub public_url: Url,
    /// Issuer URL of the `OpenID` Connect provider, from `OIDC_ISSUER_URL`.
    pub oidc_issuer_url: Url,
    /// Client ID registered with the provider, from `OIDC_CLIENT_ID`.
    pub oidc_client_id: String,
    /// Client secret for the provider, from `OIDC_CLIENT_SECRET`.
    pub oidc_client_secret: String,
    /// Group whose members administer the app, from `CRABBAGE_ADMIN_GROUP` (default `crabbage-admin`).
    pub admin_group: String,
    /// Prefix of the groups that name households, from `CRABBAGE_HOUSEHOLD_GROUP_PREFIX` (default `crabbage-household-`).
    pub household_group_prefix: String,
    /// AI settings, or `None` when AI features are disabled.
    pub ai: Option<AiConfig>,
    /// Whether recipe imports may fetch private network addresses, from
    /// `CRABBAGE_IMPORT_ALLOW_PRIVATE` (default `false`).
    pub import_allow_private: bool,
}

impl Debug for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("language", &self.language)
            .field("data_dir", &self.data_dir)
            .field("public_url", &self.public_url)
            .field("oidc_issuer_url", &self.oidc_issuer_url)
            .field("oidc_client_id", &self.oidc_client_id)
            .field("oidc_client_secret", &REDACTED)
            .field("admin_group", &self.admin_group)
            .field("household_group_prefix", &self.household_group_prefix)
            .field("ai", &self.ai)
            .field("import_allow_private", &self.import_allow_private)
            .finish()
    }
}

impl Config {
    /// Reads the configuration from the process environment.
    ///
    /// # Errors
    ///
    /// Returns every missing or invalid setting in one error.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// Reads the configuration through `lookup`, which returns `None` for unset variables.
    ///
    /// Empty values count as unset.
    ///
    /// # Errors
    ///
    /// Returns every missing or invalid setting in one error.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let mut vars = Vars::new(lookup);
        let language = vars.language();
        let data_dir = vars
            .get("CRABBAGE_DATA_DIR")
            .map_or_else(|| PathBuf::from("/data"), PathBuf::from);
        let public_url = vars.url("CRABBAGE_PUBLIC_URL");
        let oidc_issuer_url = vars.url("OIDC_ISSUER_URL");
        let oidc_client_id = vars.required("OIDC_CLIENT_ID");
        let oidc_client_secret = vars.required("OIDC_CLIENT_SECRET");
        let admin_group = vars
            .get("CRABBAGE_ADMIN_GROUP")
            .unwrap_or_else(|| "crabbage-admin".to_owned());
        let household_group_prefix = vars
            .get("CRABBAGE_HOUSEHOLD_GROUP_PREFIX")
            .unwrap_or_else(|| "crabbage-household-".to_owned());
        let ai = vars.ai();
        let import_allow_private = vars.flag("CRABBAGE_IMPORT_ALLOW_PRIVATE");
        match (
            language,
            public_url,
            oidc_issuer_url,
            oidc_client_id,
            oidc_client_secret,
        ) {
            (
                Some(language),
                Some(public_url),
                Some(oidc_issuer_url),
                Some(oidc_client_id),
                Some(oidc_client_secret),
            ) if vars.problems.is_empty() => Ok(Self {
                language,
                data_dir,
                public_url,
                oidc_issuer_url,
                oidc_client_id,
                oidc_client_secret,
                admin_group,
                household_group_prefix,
                ai,
                import_allow_private,
            }),
            _ => Err(ConfigError::new(vars.problems)),
        }
    }

    /// Whether AI features are enabled.
    #[must_use]
    pub const fn ai_enabled(&self) -> bool {
        self.ai.is_some()
    }
}
