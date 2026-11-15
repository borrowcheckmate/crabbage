//! The ingredient admin pages: the list with the add form and the search, and one ingredient's page.

mod allergens;
mod detail;
mod list;

pub(crate) use allergens::allergen_form;
pub(crate) use detail::IngredientPage;
pub(crate) use list::IngredientsPage;
