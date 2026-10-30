//! Catalog server functions: ingredients, aliases, store sections, recipe categories and the review
//! queue. Every one of them is admin-only, see ARCHITECTURE.md.

pub(crate) mod categories;
#[cfg(feature = "ssr")]
mod convert;
pub(crate) mod ingredients;
pub(crate) mod sections;
mod types;

pub(crate) use types::{CategoryRow, IngredientRow, SectionRow};
