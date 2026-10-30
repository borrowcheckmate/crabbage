//! Store records as the recipe records the pages show.

use crabbage_store::recipes::{Person, Recipe, RecipeLine, RecipeStep, RecipeSummary};

use crate::catalog::CategoryRow;

use super::rights::Rights;
use super::types::{LineRow, PersonRow, RecipeDetail, RecipeSummaryRow, StepRow};

impl From<RecipeSummary> for RecipeSummaryRow {
    fn from(summary: RecipeSummary) -> Self {
        Self {
            id: summary.id,
            title: summary.title,
            servings: summary.servings,
            owner_name: summary.owner_name,
        }
    }
}

impl From<Person> for PersonRow {
    fn from(person: Person) -> Self {
        Self {
            id: person.id,
            name: person.name,
        }
    }
}

impl From<RecipeLine> for LineRow {
    fn from(line: RecipeLine) -> Self {
        Self {
            amount: line.amount,
            unit: line.unit,
            ingredient: line.ingredient,
            note: line.note,
        }
    }
}

impl From<RecipeStep> for StepRow {
    fn from(step: RecipeStep) -> Self {
        Self {
            text: step.text,
            duration_seconds: step.duration_seconds,
        }
    }
}

/// The page record of `recipe`, with what the signed-in user may do to it.
pub(crate) fn detail(recipe: Recipe, rights: Rights) -> RecipeDetail {
    RecipeDetail {
        id: recipe.id,
        title: recipe.title,
        servings: recipe.servings,
        owner_name: recipe.owner_name,
        lines: recipe.lines.into_iter().map(LineRow::from).collect(),
        steps: recipe.steps.into_iter().map(StepRow::from).collect(),
        editors: recipe.editors.into_iter().map(PersonRow::from).collect(),
        categories: recipe
            .categories
            .into_iter()
            .map(CategoryRow::from)
            .collect(),
        image: recipe.image,
        can_edit: rights.edit,
        can_delete: rights.delete,
        can_manage_editors: rights.manage_editors,
    }
}
