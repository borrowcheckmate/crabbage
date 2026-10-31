use crate::ai::{AiClient, Prompt};
use crate::matching::Candidate;
use crate::web::Draft;

use super::{Imported, from_draft, merge, needs_ai};

/// Reads a web draft into a recipe, with the AI pass when the draft needs it and an AI client is
/// configured. An AI failure keeps the reading and sets its `ai_error`.
pub async fn complete(
    draft: &Draft,
    candidates: &[Candidate<'_>],
    ai: Option<&AiClient>,
    instance_language: &str,
) -> Imported {
    let base = from_draft(draft, candidates);
    let Some(client) = ai else {
        return base;
    };
    if !needs_ai(&base, draft.language.as_deref(), instance_language) {
        return base;
    }
    match client.complete(&prompt(draft, instance_language)).await {
        Ok(ai_draft) => merge(&ai_draft, &base, candidates),
        Err(error) => Imported {
            ai_error: Some(error.to_string()),
            ..base
        },
    }
}

fn prompt(draft: &Draft, instance_language: &str) -> Prompt {
    Prompt {
        instructions: format!(
            "Write the recipe in the language with the code {instance_language}. Use only these \
             unit symbols: g, kg, ml, l, tsp, tbsp, piece, pinch, or none. Give every step the \
             zero-based indexes of the ingredient lines it uses. Keep the amounts for the stated \
             servings."
        ),
        text: text(draft),
        images: Vec::new(),
    }
}

fn text(draft: &Draft) -> String {
    let servings = draft
        .servings
        .map_or_else(|| "unknown".to_owned(), |servings| servings.to_string());
    format!(
        "Title: {}\nServings: {servings}\n\nIngredient lines:\n{}\n\nSteps:\n{}\n",
        draft.title,
        numbered(&draft.lines),
        numbered(&draft.steps),
    )
}

fn numbered(items: &[String]) -> String {
    items
        .iter()
        .enumerate()
        .map(|(index, item)| format!("{index}. {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}
