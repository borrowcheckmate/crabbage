//! The catalog: ingredients with their aliases and allergens, store sections and recipe categories. Recipes and
//! shopping lists match against it.

pub mod allergens;
pub mod categories;
pub mod ingredients;
pub(crate) mod names;
pub mod sections;
