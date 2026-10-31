mod client;
mod error;
mod model;

pub use client::MealieClient;
pub use error::MealieError;
pub use model::{
    Category, Food, FoodAlias, FoodRef, Ingredient, Instruction, MealieUnit, Recipe, RecipeSummary,
    Tag, UnitRef,
};
