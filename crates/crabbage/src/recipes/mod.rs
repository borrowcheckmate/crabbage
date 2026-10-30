//! Recipes: the list, the recipe page, the form for new and changed recipes, and the server
//! functions behind them.

mod categories;
#[cfg(feature = "ssr")]
mod convert;
#[cfg(feature = "ssr")]
mod draft;
mod editors;
mod ingredients;
mod line_text;
mod list;
mod pages;
#[cfg(feature = "ssr")]
mod photo;
mod read;
#[cfg(feature = "ssr")]
mod rights;
mod types;
mod units;
mod write;

pub(crate) use pages::{RecipeEditPage, RecipeNewPage, RecipePage, RecipesPage};
#[cfg(feature = "ssr")]
pub(crate) use photo::routes as photo_routes;
