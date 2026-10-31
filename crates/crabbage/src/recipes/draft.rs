//! Checks a recipe form and turns it into the store's input. A row that is entirely blank is
//! skipped; every other field is checked here, so the store only sees valid recipes.

use crabbage_domain::quantity::Unit;
use crabbage_store::recipes::{LineInput, RecipeInput, StepInput};
use leptos::prelude::ServerFnError;

use super::types::{LineDraft, RecipeDraft, StepDraft};
use crate::i18n::t;

/// Reads the JSON draft a recipe form sends and checks it.
///
/// # Errors
///
/// Returns the translated reason for the first field that is not valid.
pub(crate) fn parse(json: &str) -> Result<RecipeInput, ServerFnError> {
    let draft: RecipeDraft = serde_json::from_str(json)
        .map_err(|_error| ServerFnError::new(t("recipe.error.invalid")))?;
    input(&draft)
}

fn input(draft: &RecipeDraft) -> Result<RecipeInput, ServerFnError> {
    let title = draft.title.trim();
    if title.is_empty() {
        return Err(ServerFnError::new(t("recipe.error.title")));
    }
    let servings = servings(&draft.servings)?;
    let mut lines = Vec::new();
    let mut line_index = Vec::with_capacity(draft.lines.len());
    for draft_line in &draft.lines {
        if is_blank_line(draft_line) {
            line_index.push(None);
        } else {
            line_index.push(Some(lines.len()));
            lines.push(line(draft_line)?);
        }
    }
    let steps = draft
        .steps
        .iter()
        .filter(|step| !is_blank_step(step))
        .map(|step| step_input(step, &line_index))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(RecipeInput {
        title: title.to_owned(),
        servings,
        lines,
        steps,
        category_ids: draft.categories.clone(),
    })
}

fn is_blank_line(line: &LineDraft) -> bool {
    [&line.amount, &line.unit, &line.ingredient, &line.note]
        .iter()
        .all(|field| field.trim().is_empty())
}

fn is_blank_step(step: &StepDraft) -> bool {
    step.text.trim().is_empty() && step.minutes.trim().is_empty()
}

fn servings(text: &str) -> Result<i64, ServerFnError> {
    text.trim()
        .parse::<i64>()
        .ok()
        .filter(|count| *count >= 1)
        .ok_or_else(|| ServerFnError::new(t("recipe.error.servings")))
}

fn line(draft: &LineDraft) -> Result<LineInput, ServerFnError> {
    Ok(LineInput {
        amount: amount(&draft.amount)?,
        unit: unit(&draft.unit)?,
        ingredient: optional(&draft.ingredient),
        note: optional(&draft.note),
    })
}

fn amount(text: &str) -> Result<Option<f64>, ServerFnError> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    text.parse::<f64>()
        .ok()
        .filter(|amount| amount.is_finite() && *amount >= 0.0)
        .map(Some)
        .ok_or_else(|| ServerFnError::new(t("recipe.error.amount")))
}

fn unit(text: &str) -> Result<Option<String>, ServerFnError> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    Unit::from_symbol(text)
        .map(|unit| Some(unit.symbol().to_owned()))
        .ok_or_else(|| ServerFnError::new(t("recipe.error.unit")))
}

fn optional(text: &str) -> Option<String> {
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

/// The step as the store takes it. `line_index` gives each form line's position in the stored lines,
/// or `None` for a blank line that is not stored, so the links follow the lines that are kept. A link
/// to a blank or missing line is dropped.
fn step_input(draft: &StepDraft, line_index: &[Option<usize>]) -> Result<StepInput, ServerFnError> {
    let text = draft.text.trim();
    if text.is_empty() {
        return Err(ServerFnError::new(t("recipe.error.step")));
    }
    Ok(StepInput {
        text: text.to_owned(),
        duration_seconds: minutes(&draft.minutes)?,
        line_positions: draft
            .line_positions
            .iter()
            .filter_map(|&position| line_index.get(position).copied().flatten())
            .collect(),
    })
}

fn minutes(text: &str) -> Result<Option<i64>, ServerFnError> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    text.parse::<i64>()
        .ok()
        .filter(|minutes| *minutes >= 1)
        .and_then(|minutes| minutes.checked_mul(60))
        .map(Some)
        .ok_or_else(|| ServerFnError::new(t("recipe.error.minutes")))
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn a_link_follows_its_line_past_a_blank_row_and_a_missing_link_is_dropped() {
        let json = r#"{"title":"Soup","servings":"2",
            "lines":[{"amount":"","unit":"","ingredient":"","note":""},
                     {"amount":"1","unit":"piece","ingredient":"Onion","note":""}],
            "steps":[{"text":"Chop","minutes":"","line_positions":[1,0,5]}]}"#;
        let input = parse(json).expect("the draft is valid");
        assert_eq!(input.lines.len(), 1);
        assert_eq!(input.steps[0].line_positions, [0]);
    }
}
