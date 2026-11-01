use crabbage_domain::quantity::Unit;

use crate::ai::AiPhoto;

/// A recipe ready to save: metric amounts and units, its lines matched to the catalogue, and its
/// steps linked to the lines they use.
#[derive(Debug, Clone, PartialEq)]
pub struct Imported {
    /// The recipe's title.
    pub title: String,
    /// The number of servings, when the page or the AI names one.
    pub servings: Option<u32>,
    /// The address of the recipe photo.
    pub image: Option<String>,
    /// The AI's normalized finished-dish crop on one submitted photo.
    pub photo: Option<AiPhoto>,
    /// The selected household recipe categories.
    pub categories: Vec<i64>,
    /// The ingredient lines in order.
    pub lines: Vec<ImportedLine>,
    /// The method steps in order.
    pub steps: Vec<ImportedStep>,
    /// The names of the lines that match no catalogue ingredient, once per name, in order.
    pub unmatched: Vec<String>,
    /// Why the AI pass did not complete the draft, when it was tried and failed.
    pub ai_error: Option<String>,
}

/// One ingredient line.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportedLine {
    /// The amount, in the unit.
    pub amount: Option<f64>,
    /// The unit, a metric or domain unit.
    pub unit: Option<Unit>,
    /// The ingredient's name.
    pub name: String,
    /// A note such as `finely chopped`.
    pub note: Option<String>,
    /// The catalogue ingredient the name matches, if any.
    pub ingredient_id: Option<i64>,
    /// The line as it was read.
    pub original: String,
}

/// One method step and the positions, in [`Imported::lines`], of the lines it uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedStep {
    /// The step's text.
    pub text: String,
    /// The positions of the lines the step uses, sorted, without duplicates.
    pub line_positions: Vec<usize>,
}
