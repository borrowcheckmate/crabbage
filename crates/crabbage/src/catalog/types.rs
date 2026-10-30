//! The catalog records the admin pages show. They cross the wire as JSON.

use serde::{Deserialize, Serialize};

/// An ingredient with its aliases.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct IngredientRow {
    /// The ingredient's id.
    pub(crate) id: i64,
    /// The name.
    pub(crate) name: String,
    /// The store section the ingredient is shopped in, if any.
    pub(crate) section_id: Option<i64>,
    /// Whether the ingredient waits for review.
    pub(crate) pending_review: bool,
    /// The alternative names.
    pub(crate) aliases: Vec<String>,
}

/// A store section.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SectionRow {
    /// The section's id.
    pub(crate) id: i64,
    /// The name.
    pub(crate) name: String,
    /// The place in the shopping order, from 0.
    pub(crate) position: i64,
}
