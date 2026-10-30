//! Catalog server functions: ingredients, aliases, store sections and the review queue. Every one
//! of them is admin-only, see ARCHITECTURE.md.

#[cfg(feature = "ssr")]
mod convert;
pub(crate) mod ingredients;
pub(crate) mod sections;
mod types;

pub(crate) use types::{IngredientRow, SectionRow};
