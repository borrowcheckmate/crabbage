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
mod ingredients;
mod line_text;
mod list;
mod pages;
#[cfg(feature = "ssr")]
mod photo;
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
pub(crate) use pages::{RecipeEditPage, RecipeNewPage, RecipePage, RecipesPage};
#[cfg(feature = "ssr")]
pub(crate) use photo::routes as photo_routes;
#[cfg(feature = "ssr")]
pub(crate) use rating::routes as rating_routes;
