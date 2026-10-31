use std::time::Duration;

/// A recipe read from a web page, before it is matched to the catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    /// The recipe name, trimmed and never empty.
    pub title: String,
    /// The number of servings the recipe yields.
    pub servings: Option<u32>,
    /// The address of the recipe photo.
    pub image: Option<String>,
    /// The ingredient lines, trimmed, in page order.
    pub lines: Vec<String>,
    /// The preparation steps, in page order.
    pub steps: Vec<String>,
    /// The time to prepare the ingredients.
    pub prep_time: Option<Duration>,
    /// The time the recipe spends cooking.
    pub cook_time: Option<Duration>,
    /// The time from start to finish.
    pub total_time: Option<Duration>,
}
