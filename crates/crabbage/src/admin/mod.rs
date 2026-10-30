//! The admin pages: ingredients, store sections, recipe categories and the review queue.

mod categories;
mod ingredients;
mod review;
mod sections;

pub(crate) use categories::CategoriesPage;
pub(crate) use ingredients::{IngredientPage, IngredientsPage};
pub(crate) use review::ReviewPage;
pub(crate) use sections::SectionsPage;
