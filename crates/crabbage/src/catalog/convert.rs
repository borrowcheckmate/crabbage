//! Store records as the catalog records the pages show.

use crabbage_store::catalog::ingredients::Ingredient;
use crabbage_store::catalog::sections::Section;

use super::{IngredientRow, SectionRow};

impl From<Ingredient> for IngredientRow {
    fn from(ingredient: Ingredient) -> Self {
        Self {
            id: ingredient.id,
            name: ingredient.name,
            section_id: ingredient.store_section_id,
            pending_review: ingredient.pending_review,
            aliases: ingredient.aliases,
        }
    }
}

impl From<Section> for SectionRow {
    fn from(section: Section) -> Self {
        Self {
            id: section.id,
            name: section.name,
            position: section.position,
        }
    }
}
