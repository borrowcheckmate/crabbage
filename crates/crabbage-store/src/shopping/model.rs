//! The shopping item record as the store returns it.

use serde::Serialize;
use sqlx::FromRow;

/// An item on a household's shopping list.
#[derive(Clone, Debug, PartialEq, FromRow, Serialize)]
pub struct ShoppingItem {
    /// The item's id.
    pub id: i64,
    /// The ingredient the item is for, if it has one.
    pub ingredient_id: Option<i64>,
    /// The ingredient's name, if the item has an ingredient.
    pub ingredient: Option<String>,
    /// The text of a manual item or of a recipe note without an ingredient.
    pub text: Option<String>,
    /// The amount, if the item has one.
    pub amount: Option<f64>,
    /// The unit symbol, if the item has one.
    pub unit: Option<String>,
    /// Whether the item is ticked off.
    pub checked: bool,
    /// The recipe the item came from, if it still exists.
    pub recipe_id: Option<i64>,
    /// The plan entry the item came from, if any.
    pub plan_entry_id: Option<i64>,
    /// The store section of the ingredient, if it has one.
    pub section_id: Option<i64>,
    /// The position of that section in the shopping order.
    pub section_position: Option<i64>,
}
