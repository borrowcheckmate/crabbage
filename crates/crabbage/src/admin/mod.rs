//! The admin pages: ingredients, store sections, recipe categories and the review queue.

mod categories;
mod households;
mod ingredients;
pub(crate) mod mealie;
mod review;
mod sections;
mod settings;

pub(crate) use categories::CategoriesPage;
pub(crate) use households::HouseholdsPage;
pub(crate) use ingredients::{IngredientPage, IngredientsPage};
pub(crate) use mealie::MealiePage;
pub(crate) use review::ReviewPage;
pub(crate) use sections::SectionsPage;
pub(crate) use settings::SettingsPage;
