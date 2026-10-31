//! Recipes: the list, the recipe page, the form for new and changed recipes, and the server
//! functions behind them.

mod categories;
#[cfg(feature = "ssr")]
mod convert;
mod cook;
#[cfg(feature = "ssr")]
mod draft;
mod duration;
mod editors;
#[cfg(feature = "ssr")]
mod import;
mod import_fns;
#[cfg(feature = "ssr")]
mod import_photo;
#[cfg(feature = "ssr")]
mod import_photos;
mod ingredients;
mod line_text;
mod list;
mod pages;
#[cfg(feature = "ssr")]
mod photo;
#[cfg(feature = "ssr")]
mod photo_drafts;
#[cfg(feature = "ssr")]
mod rating;
mod rating_text;
mod read;
#[cfg(feature = "ssr")]
mod rights;
mod servings;
mod types;
mod units;
mod write;

pub(crate) use cook::CookPage;
#[cfg(feature = "ssr")]
pub(crate) use import_photos::routes as import_photo_routes;
pub(crate) use pages::{RecipeEditPage, RecipeImportPage, RecipeNewPage, RecipePage, RecipesPage};
#[cfg(feature = "ssr")]
pub(crate) use photo::routes as photo_routes;
#[cfg(feature = "ssr")]
pub(crate) use photo_drafts::PhotoDrafts;
#[cfg(feature = "ssr")]
pub(crate) use rating::routes as rating_routes;
