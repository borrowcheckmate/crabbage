//! Translated text: the languages the app speaks and the messages each one maps keys to.

mod context;
mod language;
mod messages;
#[cfg(test)]
mod tests;

#[cfg(feature = "hydrate")]
pub(crate) use context::document_language;
pub(crate) use context::{current_language, messages, t, t_with};
pub use language::Language;
pub(crate) use messages::Messages;
