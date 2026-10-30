//! The signals behind the recipe form. Each field is a signal, so rows can be added and removed,
//! and the whole draft is read as JSON when the form is sent.

use leptos::prelude::*;

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
                })
                .collect(),
        })
    }

    fn from_wire(draft: &RecipeDraft) -> Self {
        let form = Self {
            title: RwSignal::new(draft.title.clone()),
            servings: RwSignal::new(draft.servings.clone()),
            lines: RwSignal::new(Vec::new()),
            steps: RwSignal::new(Vec::new()),
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

    /// Removes the ingredient row with `key`.
    pub(crate) fn remove_line(&self, key: usize) {
        self.lines.update(|rows| rows.retain(|row| row.key != key));
    }

    /// Adds a step row at the end, filled with `step`.
    pub(crate) fn add_step(&self, step: StepDraft) {
        let row = StepSignals {
            key: self.next_key(),
            text: RwSignal::new(step.text),
            minutes: RwSignal::new(step.minutes),
        };
        self.steps.update(|rows| rows.push(row));
    }

    /// Removes the step row with `key`.
    pub(crate) fn remove_step(&self, key: usize) {
        self.steps.update(|rows| rows.retain(|row| row.key != key));
    }

    /// The draft as the form currently shows it.
    pub(crate) fn current(&self) -> RecipeDraft {
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
                })
                .collect(),
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
