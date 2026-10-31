//! Turns a recipe read from a web page into the store's input, so it is saved like a typed one.
//! Ingredient names that match no catalogue ingredient are created pending review by the store.

use crabbage_import::pipeline::Imported;
use crabbage_store::recipes::{LineInput, RecipeInput, StepInput};

/// The store input for `imported`, with `default_servings` when the recipe names none. A step's
/// line position that is not a line of the recipe is dropped.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "the import is not yet wired to a page")
)]
#[must_use]
pub fn recipe_input(imported: &Imported, default_servings: i64) -> RecipeInput {
    let lines: Vec<LineInput> = imported
        .lines
        .iter()
        .map(|line| LineInput {
            amount: line.amount,
            unit: line.unit.map(|unit| unit.symbol().to_owned()),
            ingredient: Some(line.name.clone()),
            note: line.note.clone(),
        })
        .collect();
    let line_count = lines.len();
    let steps = imported
        .steps
        .iter()
        .map(|step| StepInput {
            text: step.text.clone(),
            duration_seconds: None,
            line_positions: step
                .line_positions
                .iter()
                .copied()
                .filter(|position| *position < line_count)
                .collect(),
        })
        .collect();
    RecipeInput {
        title: imported.title.clone(),
        servings: imported.servings.map_or(default_servings, i64::from),
        lines,
        steps,
        category_ids: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use crabbage_domain::quantity::Unit;
    use crabbage_import::pipeline::{ImportedLine, ImportedStep};
    use crabbage_store::catalog::ingredients;
    use crabbage_store::recipes::create;

    use super::*;

    fn line(name: &str, amount: f64, unit: Option<Unit>) -> ImportedLine {
        ImportedLine {
            amount: Some(amount),
            unit,
            name: name.to_owned(),
            note: None,
            ingredient_id: None,
            original: name.to_owned(),
        }
    }

    fn recipe(lines: Vec<ImportedLine>, steps: Vec<ImportedStep>) -> Imported {
        Imported {
            title: "Okra soup".to_owned(),
            servings: None,
            image: None,
            lines,
            steps,
            unmatched: Vec::new(),
            ai_error: None,
        }
    }

    #[test]
    fn units_and_names_map_and_missing_servings_take_the_default() {
        let imported = recipe(
            vec![line("Okra", 200.0, Some(Unit::Gram))],
            vec![ImportedStep {
                text: "Cook.".to_owned(),
                line_positions: vec![0],
            }],
        );
        let input = recipe_input(&imported, 4);
        assert_eq!(input.servings, 4);
        assert_eq!(input.lines[0].unit.as_deref(), Some("g"));
        assert_eq!(input.lines[0].ingredient.as_deref(), Some("Okra"));
        assert_eq!(input.lines[0].amount, Some(200.0));
        assert_eq!(input.category_ids, Vec::<i64>::new());
        assert_eq!(input.steps[0].duration_seconds, None);
    }

    #[test]
    fn step_positions_outside_the_lines_are_dropped() {
        let imported = recipe(
            vec![line("Okra", 1.0, None), line("Salt", 1.0, None)],
            vec![ImportedStep {
                text: "Cook.".to_owned(),
                line_positions: vec![0, 1, 2, 7],
            }],
        );
        assert_eq!(
            recipe_input(&imported, 2).steps[0].line_positions,
            vec![0, 1]
        );
    }

    #[tokio::test]
    async fn only_the_new_ingredient_is_saved_pending_review() {
        let dir = std::env::temp_dir().join(format!("crabbage-import-{}", std::process::id()));
        drop(std::fs::remove_dir_all(&dir));
        let pool = crabbage_store::open(&dir).await.expect("database opens");
        let owner = crabbage_store::users::upsert_user(&pool, "subject", "Cook")
            .await
            .expect("user");
        ingredients::create(&pool, "Salt")
            .await
            .expect("ingredient");
        let imported = recipe(
            vec![
                line("Salt", 1.0, Some(Unit::Teaspoon)),
                line("Okra", 200.0, Some(Unit::Gram)),
            ],
            vec![ImportedStep {
                text: "Cook.".to_owned(),
                line_positions: vec![0, 1],
            }],
        );
        create(&pool, owner, &recipe_input(&imported, 2), 0)
            .await
            .expect("recipe saved");
        let pending: Vec<String> = ingredients::list_pending(&pool)
            .await
            .expect("pending list")
            .into_iter()
            .map(|ingredient| ingredient.name)
            .collect();
        drop(std::fs::remove_dir_all(&dir));
        assert_eq!(pending, vec!["Okra".to_owned()]);
    }
}
