//! The household's shopping list: recipe lines added to it, manual items, checking and clearing.

mod model;
mod read;
mod write;

pub use model::ShoppingItem;
pub use read::list;
pub use write::{add_recipe, add_text, clear_checked, edit_text, remove, set_checked};
