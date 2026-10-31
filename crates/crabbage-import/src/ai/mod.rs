//! AI draft client: sends a recipe's text and photos to an OpenAI-compatible chat endpoint and
//! reads the recipe draft it answers with.

mod client;
mod draft;
mod error;
mod request;

pub use client::AiClient;
pub use draft::{AiDraft, AiLine, AiPhoto, AiStep, parse};
pub use error::AiError;
pub use request::{Image, Prompt};
