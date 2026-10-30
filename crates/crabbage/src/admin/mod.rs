//! The admin pages: ingredients, store sections and the review queue.

mod ingredients;
mod notice;
mod review;
mod sections;

pub(crate) use ingredients::IngredientsPage;
pub(crate) use review::ReviewPage;
pub(crate) use sections::SectionsPage;
