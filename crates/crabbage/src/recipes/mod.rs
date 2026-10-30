//! Recipes: the list, the recipe page, the form for new and changed recipes, and the server
//! functions behind them.

#[cfg(feature = "ssr")]
mod convert;
#[cfg(feature = "ssr")]
mod draft;
mod editors;
mod ingredients;
mod list;
mod pages;
mod read;
#[cfg(feature = "ssr")]
mod rights;
mod types;
mod units;
mod write;

pub(crate) use pages::{RecipeEditPage, RecipeNewPage, RecipePage, RecipesPage};
