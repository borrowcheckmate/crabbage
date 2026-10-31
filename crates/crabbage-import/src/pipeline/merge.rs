use crabbage_domain::quantity::Unit;

use crate::ai::{AiDraft, AiLine};
use crate::lines::measure_of;
use crate::matching::{Candidate, match_all, match_ingredient};

use super::{Imported, ImportedLine, ImportedStep};

/// Merges the AI's draft into the reading of the page.
///
/// The AI's title, lines and steps replace the reading's. The servings fall back to the reading's and
/// the photo is always the reading's. Every amount is metric afterwards. A line keeps the AI's
/// ingredient id only when it is one of the candidates' ids, otherwise it is matched by name again.
#[must_use]
pub fn merge(ai: &AiDraft, base: &Imported, candidates: &[Candidate<'_>]) -> Imported {
    let lines: Vec<ImportedLine> = ai
        .lines
        .iter()
        .map(|line| merged_line(line, candidates))
        .collect();
    let steps = ai
        .steps
        .iter()
        .map(|step| ImportedStep {
            text: step.text.clone(),
            line_positions: sorted_unique(&step.lines),
        })
        .collect();
    let names: Vec<&str> = lines
        .iter()
        .filter(|line| line.ingredient_id.is_none())
        .map(|line| line.name.as_str())
        .collect();
    Imported {
        title: ai.title.clone(),
        servings: ai.servings.or(base.servings),
        image: base.image.clone(),
        photo: ai.photo,
        unmatched: match_all(&names, candidates).unmatched,
        lines,
        steps,
        ai_error: None,
    }
}

fn merged_line(line: &AiLine, candidates: &[Candidate<'_>]) -> ImportedLine {
    let name = line.name.trim().to_owned();
    let (amount, unit) = metric(line.amount, line.unit.as_deref());
    let ingredient_id = line
        .ingredient_id
        .filter(|id| candidates.iter().any(|candidate| candidate.id == *id))
        .or_else(|| match_ingredient(&name, candidates));
    let note = line
        .note
        .as_deref()
        .map(str::trim)
        .filter(|note| !note.is_empty())
        .map(str::to_owned);
    ImportedLine {
        original: render(amount, unit, &name, note.as_deref()),
        amount,
        unit,
        name,
        note,
        ingredient_id,
    }
}

/// The amount and unit in metric. A unit the domain knows keeps its amount. An imperial or other
/// measure is converted, and a measure without an amount has nothing to convert, so it is dropped.
fn metric(amount: Option<f64>, symbol: Option<&str>) -> (Option<f64>, Option<Unit>) {
    let Some(symbol) = symbol else {
        return (amount, None);
    };
    if let Some(unit) = Unit::from_symbol(symbol) {
        return (amount, Some(unit));
    }
    let Some((amount, measure)) = amount.zip(measure_of(symbol)) else {
        return (None, None);
    };
    measure.quantity(amount).map_or((None, None), |quantity| {
        (Some(quantity.amount()), Some(quantity.unit()))
    })
}

fn render(amount: Option<f64>, unit: Option<Unit>, name: &str, note: Option<&str>) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(amount) = amount {
        parts.push(amount.to_string());
    }
    if let Some(unit) = unit.filter(|unit| *unit != Unit::ToTaste) {
        parts.push(unit.symbol().to_owned());
    }
    parts.push(name.to_owned());
    if unit == Some(Unit::ToTaste) {
        parts.push(Unit::ToTaste.symbol().to_owned());
    }
    let text = parts.join(" ");
    match note {
        Some(note) => format!("{text}, {note}"),
        None => text,
    }
}

fn sorted_unique(positions: &[usize]) -> Vec<usize> {
    let mut sorted = positions.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    sorted
}
