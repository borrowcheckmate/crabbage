use crate::ai::{AiClient, Prompt};
use crate::matching::Candidate;
use crate::web::Draft;

use super::{Imported, from_draft, merge};

/// Reads a web draft into a recipe, with an AI pass whenever an AI client is configured. An AI
/// failure keeps the reading and sets its `ai_error`.
pub async fn complete(
    draft: &Draft,
    candidates: &[Candidate<'_>],
    categories: &[(i64, &str)],
    ai: Option<&AiClient>,
    instance_language: &str,
) -> Imported {
    let base = from_draft(draft, candidates);
    let Some(client) = ai else {
        return base;
    };
    match client
        .complete(&prompt(draft, instance_language, candidates, categories))
        .await
    {
        Ok(ai_draft) => merge(
            &ai_draft,
            &base,
            candidates,
            &categories.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        ),
        Err(error) => Imported {
            ai_error: Some(error.to_string()),
            ..base
        },
    }
}

fn prompt(
    draft: &Draft,
    instance_language: &str,
    candidates: &[Candidate<'_>],
    categories: &[(i64, &str)],
) -> Prompt {
    Prompt {
        instructions: instructions(instance_language),
        text: text(draft, candidates, categories),
        images: Vec::new(),
    }
}

/// The instructions every AI pass starts with.
pub(super) fn instructions(instance_language: &str) -> String {
    format!(
        "Write the recipe in the language with the code {instance_language}. Ingredient name \
         must be only the ingredient as bought, singular or plural as written, without preparation, \
         temperature, size, cut, quantity words, or 'to taste'/'of choice'; put all such text in \
         note. Example: 'slagroom (koud)' has name 'slagroom', note 'koud'; '8 dikke sneetjes \
         briochebrood' has amount 8, unit piece, name 'briochebrood', note 'dikke sneetjes'; \
         'rood fruit naar keuze' has name 'rood fruit', note 'naar keuze'. Set ingredient_id \
         when a provided candidate clearly matches. Choose categories only from the provided \
         category ids. Use only these \
         unit symbols: g, kg, ml, l, tsp, tbsp, piece, pinch, or none. Give every step the \
         zero-based indexes of the ingredient lines it uses. Keep the amounts for the stated \
         servings."
    )
}

pub(super) fn known_ingredients_and_categories(
    candidates: &[Candidate<'_>],
    categories: &[(i64, &str)],
) -> String {
    format!(
        "Known ingredients (id: name):\n{}\n\nRecipe categories (id: name):\n{}",
        candidates
            .iter()
            .take(400)
            .map(|candidate| format!("{}: {}", candidate.id, candidate.name))
            .collect::<Vec<_>>()
            .join("\n"),
        categories
            .iter()
            .map(|(id, name)| format!("{id}: {name}"))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn text(draft: &Draft, candidates: &[Candidate<'_>], categories: &[(i64, &str)]) -> String {
    let servings = draft
        .servings
        .map_or_else(|| "unknown".to_owned(), |servings| servings.to_string());
    format!(
        "Title: {}\nServings: {servings}\n\n{}\n\nIngredient lines:\n{}\n\nSteps:\n{}\n",
        draft.title,
        known_ingredients_and_categories(candidates, categories),
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
