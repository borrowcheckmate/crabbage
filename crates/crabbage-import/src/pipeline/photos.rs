use crate::ai::{AiClient, AiError, Image, Prompt};
use crate::matching::Candidate;

use super::run::instructions;
use super::{Imported, merge};

/// Reads a recipe from photos of a cookbook page or recipe card with the AI. The imported recipe
/// has no image address; `photo` carries a normalized crop when the AI found a finished-dish photo.
///
/// # Errors
///
/// Returns the [`AiError`] when the AI cannot produce a usable draft.
pub async fn from_photos(
    images: Vec<Image>,
    candidates: &[Candidate<'_>],
    client: &AiClient,
    instance_language: &str,
) -> Result<Imported, AiError> {
    let prompt = Prompt {
        instructions: format!(
            "{} Read the recipe from the photos. Choose a clear finished-dish photo when present, +             prefer it over step photos, and return null rather than selecting the whole page. +             Return null for text-only pages, ads, or pages without a finished-dish photo.",
            instructions(instance_language)
        ),
        text: String::new(),
        images,
    };
    let ai = client.complete(&prompt).await?;
    let base = Imported {
        title: String::new(),
        servings: None,
        image: None,
        photo: None,
        lines: Vec::new(),
        steps: Vec::new(),
        unmatched: Vec::new(),
        ai_error: None,
    };
    Ok(merge(&ai, &base, candidates))
}
