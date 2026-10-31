//! The signals behind the recipe form. Each field is a signal, so rows can be added and removed,
//! and the whole draft is read as JSON when the form is sent.

use leptos::prelude::*;

use crate::i18n::messages;
use crate::recipes::line_text::line_text;
use crate::recipes::types::{LineDraft, LineRow, RecipeDetail, RecipeDraft, StepDraft};

/// An ingredient row of the form.
#[derive(Clone, Copy)]
pub(crate) struct LineSignals {
    /// Tells the rows apart while they are added and removed.
    pub(crate) key: usize,
    /// The amount, as typed.
    pub(crate) amount: RwSignal<String>,
    /// The unit symbol, or empty.
    pub(crate) unit: RwSignal<String>,
    /// The ingredient's name, as typed.
    pub(crate) ingredient: RwSignal<String>,
    /// The note, as typed.
    pub(crate) note: RwSignal<String>,
}

/// A step row of the form.
#[derive(Clone, Copy)]
pub(crate) struct StepSignals {
    /// Tells the rows apart while they are added and removed.
    pub(crate) key: usize,
    /// The instruction, as typed.
    pub(crate) text: RwSignal<String>,
    /// The minutes, as typed.
    pub(crate) minutes: RwSignal<String>,
    /// The keys of the ingredient rows this step uses. Keys, not positions, so a link follows its
    /// row when other rows are removed.
    pub(crate) lines: RwSignal<Vec<usize>>,
}

/// The whole recipe form.
#[derive(Clone, Copy)]
pub(crate) struct Draft {
    /// The title, as typed.
    pub(crate) title: RwSignal<String>,
    /// The servings, as typed.
    pub(crate) servings: RwSignal<String>,
    /// The ingredient rows, in order.
    pub(crate) lines: RwSignal<Vec<LineSignals>>,
    /// The step rows, in order.
    pub(crate) steps: RwSignal<Vec<StepSignals>>,
    /// The ids of the categories the recipe is filed under.
    pub(crate) categories: RwSignal<Vec<i64>>,
    next_key: RwSignal<usize>,
}

impl Draft {
    /// The draft of a new recipe: no rows, and two servings to start with.
    pub(crate) fn blank() -> Self {
        Self::from_wire(&RecipeDraft {
            servings: "2".to_owned(),
            ..RecipeDraft::default()
        })
    }

    /// The draft of an existing recipe, for the edit page.
    pub(crate) fn of(recipe: &RecipeDetail) -> Self {
        Self::from_wire(&RecipeDraft {
            title: recipe.title.clone(),
            servings: recipe.servings.to_string(),
            lines: recipe.lines.iter().map(line_draft).collect(),
            steps: recipe
                .steps
                .iter()
                .map(|step| StepDraft {
                    text: step.text.clone(),
                    minutes: step
                        .duration_seconds
                        .map(|seconds| (seconds / 60).to_string())
                        .unwrap_or_default(),
                    line_positions: step.line_positions.clone(),
                })
                .collect(),
            categories: recipe
                .categories
                .iter()
                .map(|category| category.id)
                .collect(),
        })
    }

    fn from_wire(draft: &RecipeDraft) -> Self {
        let form = Self {
            title: RwSignal::new(draft.title.clone()),
            servings: RwSignal::new(draft.servings.clone()),
            lines: RwSignal::new(Vec::new()),
            steps: RwSignal::new(Vec::new()),
            categories: RwSignal::new(draft.categories.clone()),
            next_key: RwSignal::new(0),
        };
        for line in &draft.lines {
            form.add_line(line.clone());
        }
        for step in &draft.steps {
            form.add_step(step.clone());
        }
        form
    }

    /// Adds an ingredient row at the end, filled with `line`.
    pub(crate) fn add_line(&self, line: LineDraft) {
        let row = LineSignals {
            key: self.next_key(),
            amount: RwSignal::new(line.amount),
            unit: RwSignal::new(line.unit),
            ingredient: RwSignal::new(line.ingredient),
            note: RwSignal::new(line.note),
        };
        self.lines.update(|rows| rows.push(row));
    }

    /// Removes the ingredient row with `key`, and the links to it.
    pub(crate) fn remove_line(&self, key: usize) {
        self.lines.update(|rows| rows.retain(|row| row.key != key));
        self.steps.with_untracked(|rows| {
            for row in rows {
                row.lines.update(|keys| keys.retain(|&other| other != key));
            }
        });
    }

    /// Adds a step row at the end, filled with `step`. Its links are the lines at its positions.
    pub(crate) fn add_step(&self, step: StepDraft) {
        let keys = self.line_keys();
        let linked = step
            .line_positions
            .iter()
            .filter_map(|&position| keys.get(position).copied())
            .collect();
        let row = StepSignals {
            key: self.next_key(),
            text: RwSignal::new(step.text),
            minutes: RwSignal::new(step.minutes),
            lines: RwSignal::new(linked),
        };
        self.steps.update(|rows| rows.push(row));
    }

    /// The keys of the ingredient rows, in order.
    pub(crate) fn line_keys(&self) -> Vec<usize> {
        self.lines
            .with_untracked(|rows| rows.iter().map(|row| row.key).collect())
    }

    /// Removes the step row with `key`.
    pub(crate) fn remove_step(&self, key: usize) {
        self.steps.update(|rows| rows.retain(|row| row.key != key));
    }

    /// Files the recipe under category `id`, or takes it out when `on` is false.
    pub(crate) fn set_category(&self, id: i64, on: bool) {
        self.categories.update(|ids| {
            ids.retain(|&other| other != id);
            if on {
                ids.push(id);
            }
        });
    }

    /// The draft as the form currently shows it. Each step's links are the positions of its lines.
    pub(crate) fn current(&self) -> RecipeDraft {
        let keys = self.line_keys();
        RecipeDraft {
            title: self.title.get(),
            servings: self.servings.get(),
            lines: self
                .lines
                .get()
                .iter()
                .map(|row| LineDraft {
                    amount: row.amount.get(),
                    unit: row.unit.get(),
                    ingredient: row.ingredient.get(),
                    note: row.note.get(),
                })
                .collect(),
            steps: self
                .steps
                .get()
                .iter()
                .map(|row| StepDraft {
                    text: row.text.get(),
                    minutes: row.minutes.get(),
                    line_positions: positions_of(&keys, &row.lines.get()),
                })
                .collect(),
            categories: self.categories.get(),
        }
    }

    /// The draft as JSON, for the hidden field the form sends.
    pub(crate) fn json(&self) -> String {
        serde_json::to_string(&self.current()).unwrap_or_default()
    }

    fn next_key(&self) -> usize {
        let key = self.next_key.get_untracked();
        self.next_key.set(key + 1);
        key
    }
}

/// The positions in `keys` of the keys in `linked`, in order. A linked key that is no longer in `keys`
/// has no position and is dropped.
pub(crate) fn positions_of(keys: &[usize], linked: &[usize]) -> Vec<usize> {
    keys.iter()
        .enumerate()
        .filter(|(_, key)| linked.contains(key))
        .map(|(position, _)| position)
        .collect()
}

impl LineSignals {
    /// The line's readable text, as the cook sees it: `250 g Flour, sifted`.
    pub(crate) fn text(&self) -> String {
        let line = LineRow {
            amount: self.amount.get().trim().parse().ok(),
            unit: non_empty(&self.unit.get()),
            ingredient: non_empty(&self.ingredient.get()),
            note: non_empty(&self.note.get()),
        };
        line_text(&line, messages())
    }
}

fn non_empty(text: &str) -> Option<String> {
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

impl StepSignals {
    /// Links this step to the ingredient row with `key`, or unlinks it when `on` is false.
    pub(crate) fn set_line(&self, key: usize, on: bool) {
        self.lines.update(|keys| {
            keys.retain(|&other| other != key);
            if on {
                keys.push(key);
            }
        });
    }
}

fn line_draft(line: &LineRow) -> LineDraft {
    LineDraft {
        amount: line
            .amount
            .map(|amount| amount.to_string())
            .unwrap_or_default(),
        unit: line.unit.clone().unwrap_or_default(),
        ingredient: line.ingredient.clone().unwrap_or_default(),
        note: line.note.clone().unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::positions_of;

    #[test]
    fn links_become_the_positions_of_their_lines_in_line_order() {
        assert_eq!(positions_of(&[10, 11, 12], &[12, 10, 99]), [0, 2]);
    }

    #[test]
    fn a_removed_line_drops_its_link_and_the_later_links_shift_down() {
        assert_eq!(positions_of(&[10, 12], &[10, 11, 12]), [0, 1]);
        assert_eq!(positions_of(&[11, 12], &[10, 12]), [1]);
    }
}
