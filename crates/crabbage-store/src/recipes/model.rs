//! The recipe records: what a caller writes and what the store returns.

use sqlx::FromRow;

use crate::catalog::categories::Category;

/// A recipe to create or replace: its title, servings, ingredient lines and steps.
#[derive(Clone, Debug, PartialEq)]
pub struct RecipeInput {
    /// The title.
    pub title: String,
    /// The number of servings the amounts are for.
    pub servings: i64,
    /// The ingredient lines, in order.
    pub lines: Vec<LineInput>,
    /// The steps, in order.
    pub steps: Vec<StepInput>,
    /// The categories the recipe is filed under. Unknown ids are ignored.
    pub category_ids: Vec<i64>,
}

/// An ingredient line to write.
#[derive(Clone, Debug, PartialEq)]
pub struct LineInput {
    /// The amount, if the line has one.
    pub amount: Option<f64>,
    /// The unit symbol, if the line has one.
    pub unit: Option<String>,
    /// The name of the ingredient. A name that matches no ingredient or alias creates an ingredient
    /// waiting for review.
    pub ingredient: Option<String>,
    /// A note for the cook, such as "finely chopped".
    pub note: Option<String>,
}

/// A step to write.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepInput {
    /// The instruction.
    pub text: String,
    /// How long the step takes, in seconds, if it has a duration.
    pub duration_seconds: Option<i64>,
    /// The positions, in [`RecipeInput::lines`], of the lines this step uses.
    pub line_positions: Vec<usize>,
}

/// A recipe as the store returns it: its lines and steps in order, its owner and its editors.
#[derive(Clone, Debug, PartialEq)]
pub struct Recipe {
    /// The recipe's id.
    pub id: i64,
    /// The title.
    pub title: String,
    /// The number of servings the amounts are for.
    pub servings: i64,
    /// The id of the user who created the recipe.
    pub owner_id: i64,
    /// The name of the owner.
    pub owner_name: String,
    /// The users who may edit the recipe, ordered by name.
    pub editors: Vec<Person>,
    /// The ingredient lines, in order.
    pub lines: Vec<RecipeLine>,
    /// The steps, in order.
    pub steps: Vec<RecipeStep>,
    /// The categories the recipe is filed under, ordered by name.
    pub categories: Vec<Category>,
}

/// An ingredient line as the store returns it.
#[derive(Clone, Debug, PartialEq, FromRow)]
pub struct RecipeLine {
    /// The line's id, which steps link to.
    pub id: i64,
    /// The amount, if the line has one.
    pub amount: Option<f64>,
    /// The unit symbol, if the line has one.
    pub unit: Option<String>,
    /// The name of the ingredient, if the line has one.
    pub ingredient: Option<String>,
    /// The note for the cook.
    pub note: Option<String>,
}

/// A step as the store returns it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecipeStep {
    /// The instruction.
    pub text: String,
    /// How long the step takes, in seconds, if it has a duration.
    pub duration_seconds: Option<i64>,
    /// The ids of the lines this step uses, in line order.
    pub line_ids: Vec<i64>,
}

/// A recipe as the list shows it.
#[derive(Clone, Debug, PartialEq, Eq, FromRow)]
pub struct RecipeSummary {
    /// The recipe's id.
    pub id: i64,
    /// The title.
    pub title: String,
    /// The number of servings the amounts are for.
    pub servings: i64,
    /// The name of the owner.
    pub owner_name: String,
}

/// A user's id and name.
#[derive(Clone, Debug, PartialEq, Eq, FromRow)]
pub struct Person {
    /// The user's id.
    pub id: i64,
    /// The user's name from the latest login.
    pub name: String,
}
