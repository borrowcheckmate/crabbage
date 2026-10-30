mod error;
mod settings;
#[cfg(test)]
mod tests;
mod vars;

pub use error::ConfigError;
pub use settings::{AiConfig, Config, Language};
