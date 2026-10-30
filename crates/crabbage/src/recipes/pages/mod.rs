//! The recipe pages: the list, one recipe, and the form for new and changed recipes.

mod detail;
mod editor_list;
mod form;
mod list;
mod route;

pub(crate) use detail::RecipePage;
pub(crate) use form::{RecipeEditPage, RecipeNewPage};
pub(crate) use list::RecipesPage;
