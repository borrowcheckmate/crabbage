//! The form for new and changed recipes: the signals behind it, its fields and its two pages.

mod draft;
mod fields;
mod page;

pub(super) use draft::{Draft, LineSignals, StepSignals};
pub(crate) use page::{RecipeEditPage, RecipeNewPage};
