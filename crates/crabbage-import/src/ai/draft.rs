use crabbage_domain::quantity::Unit;
use serde::Deserialize;

use super::AiError;

/// A recipe draft as the model wrote it. Ingredient lines are referred to by their index.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AiDraft {
    /// The recipe's title; never blank.
    pub title: String,
    /// The number of servings, when the recipe names one.
    #[serde(default)]
    pub servings: Option<u32>,
    /// The ingredient lines in order; at least one.
    #[serde(default)]
    pub lines: Vec<AiLine>,
    /// The method steps in order; at least one.
    #[serde(default)]
    pub steps: Vec<AiStep>,
    /// The preparation time in minutes.
    #[serde(default)]
    pub prep_minutes: Option<u32>,
    /// The cooking time in minutes.
    #[serde(default)]
    pub cook_minutes: Option<u32>,
}

/// One ingredient line of a draft.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AiLine {
    /// The amount; never negative.
    #[serde(default)]
    pub amount: Option<f64>,
    /// A unit symbol the domain knows, such as `g` or `tsp`.
    #[serde(default)]
    pub unit: Option<String>,
    /// The ingredient's name; never blank.
    pub name: String,
    /// A note such as `diced`.
    #[serde(default)]
    pub note: Option<String>,
    /// The catalogue ingredient the model matched, if any.
    #[serde(default)]
    pub ingredient_id: Option<i64>,
}

/// One method step of a draft.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AiStep {
    /// The step's text; never blank.
    pub text: String,
    /// The indexes of the ingredient lines the step uses; each must exist.
    #[serde(default)]
    pub lines: Vec<usize>,
}

/// Reads a draft from the model's answer text, which may sit inside a JSON code fence.
///
/// # Errors
///
/// [`AiError::Invalid`] with the reason when the text is not the draft shape or breaks a rule.
pub fn parse(content: &str) -> Result<AiDraft, AiError> {
    let draft: AiDraft = serde_json::from_str(unfence(content))
        .map_err(|error| invalid(format!("the draft is not the expected JSON: {error}")))?;
    validate(&draft)?;
    Ok(draft)
}

fn unfence(content: &str) -> &str {
    let trimmed = content.trim();
    trimmed.strip_prefix("```").map_or(trimmed, |rest| {
        let rest = rest.strip_prefix("json").unwrap_or(rest);
        rest.trim_end().strip_suffix("```").unwrap_or(rest).trim()
    })
}

fn validate(draft: &AiDraft) -> Result<(), AiError> {
    if draft.title.trim().is_empty() {
        return Err(invalid("the title is blank".to_owned()));
    }
    if draft.lines.is_empty() {
        return Err(invalid("the draft has no ingredient lines".to_owned()));
    }
    if draft.steps.is_empty() {
        return Err(invalid("the draft has no steps".to_owned()));
    }
    for (index, line) in draft.lines.iter().enumerate() {
        validate_line(index + 1, line)?;
    }
    for (index, step) in draft.steps.iter().enumerate() {
        validate_step(index + 1, step, draft.lines.len())?;
    }
    Ok(())
}

fn validate_line(number: usize, line: &AiLine) -> Result<(), AiError> {
    if line.name.trim().is_empty() {
        return Err(invalid(format!("ingredient {number} has no name")));
    }
    if line
        .amount
        .is_some_and(|amount| !amount.is_finite() || amount < 0.0)
    {
        return Err(invalid(format!(
            "ingredient {number} has an invalid amount"
        )));
    }
    match line.unit.as_deref() {
        Some(symbol) if Unit::from_symbol(symbol).is_none() => Err(invalid(format!(
            "ingredient {number} has an unknown unit: {symbol}"
        ))),
        _ => Ok(()),
    }
}

fn validate_step(number: usize, step: &AiStep, line_count: usize) -> Result<(), AiError> {
    if step.text.trim().is_empty() {
        return Err(invalid(format!("step {number} has no text")));
    }
    step.lines
        .iter()
        .find(|&&line| line >= line_count)
        .map_or(Ok(()), |line| {
            Err(invalid(format!(
                "step {number} refers to ingredient {}, which does not exist",
                line + 1
            )))
        })
}

const fn invalid(reason: String) -> AiError {
    AiError::Invalid(reason)
}
