//! The recipe records the pages show. They cross the wire as JSON, and a recipe form sends its
//! draft the same way.

use serde::{Deserialize, Serialize};

/// A recipe as the list shows it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RecipeSummaryRow {
    /// The recipe's id.
    pub(crate) id: i64,
    /// The title.
    pub(crate) title: String,
    /// The number of servings the amounts are for.
    pub(crate) servings: i64,
    /// The name of the owner.
    pub(crate) owner_name: String,
}

/// A recipe as its page shows it, with what the signed-in user may do to it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct RecipeDetail {
    /// The recipe's id.
    pub(crate) id: i64,
    /// The title.
    pub(crate) title: String,
    /// The number of servings the amounts are for.
    pub(crate) servings: i64,
    /// The name of the owner.
    pub(crate) owner_name: String,
    /// The ingredient lines, in order.
    pub(crate) lines: Vec<LineRow>,
    /// The steps, in order.
    pub(crate) steps: Vec<StepRow>,
    /// The users who may edit the recipe.
    pub(crate) editors: Vec<PersonRow>,
    /// Whether the signed-in user may change the recipe.
    pub(crate) can_edit: bool,
    /// Whether the signed-in user may delete the recipe.
    pub(crate) can_delete: bool,
    /// Whether the signed-in user may add and remove editors.
    pub(crate) can_manage_editors: bool,
}

/// An ingredient line as its page shows it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct LineRow {
    /// The amount, if the line has one.
    pub(crate) amount: Option<f64>,
    /// The unit symbol, if the line has one.
    pub(crate) unit: Option<String>,
    /// The ingredient's name, if the line has one.
    pub(crate) ingredient: Option<String>,
    /// The note for the cook.
    pub(crate) note: Option<String>,
}

/// A step as its page shows it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StepRow {
    /// The instruction.
    pub(crate) text: String,
    /// How long the step takes, in seconds, if it has a duration.
    pub(crate) duration_seconds: Option<i64>,
}

/// A user's id and name, to pick an editor from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PersonRow {
    /// The user's id.
    pub(crate) id: i64,
    /// The user's name.
    pub(crate) name: String,
}

/// A recipe form as it is sent. Every field is text as typed, so the server checks each one. A
/// missing field is blank, and blank fields are refused like any other.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct RecipeDraft {
    /// The title.
    pub(crate) title: String,
    /// The number of servings, as typed.
    pub(crate) servings: String,
    /// The ingredient lines, in order.
    pub(crate) lines: Vec<LineDraft>,
    /// The steps, in order.
    pub(crate) steps: Vec<StepDraft>,
}

/// An ingredient line of a recipe form.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct LineDraft {
    /// The amount, as typed. Empty for none.
    pub(crate) amount: String,
    /// The unit symbol, or empty for none.
    pub(crate) unit: String,
    /// The ingredient's name, as typed.
    pub(crate) ingredient: String,
    /// The note, as typed.
    pub(crate) note: String,
}

/// A step of a recipe form.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StepDraft {
    /// The instruction, as typed.
    pub(crate) text: String,
    /// The minutes, as typed. Empty for no duration.
    pub(crate) minutes: String,
}
