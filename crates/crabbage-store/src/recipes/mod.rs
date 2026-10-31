//! Recipes: their lines and steps, their editors, their ratings and favourites, and reading and
//! deleting them.

mod editors;
mod image;
mod mealie;
mod model;
mod read;
mod standing;
mod write;

pub use editors::{add_editor, remove_editor};
pub use image::set_image;
pub use mealie::{create_mealie, imported_mealie_ids};
pub use model::{
    LineInput, Person, Recipe, RecipeInput, RecipeLine, RecipeStep, RecipeSummary, StepInput,
};
pub use read::{get, list, list_users};
pub use standing::{rate, set_favourite};
pub use write::{create, delete, update};
