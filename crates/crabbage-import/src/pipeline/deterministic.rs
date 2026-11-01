use crate::lines::{ParsedLine, parse_line};
use crate::matching::{Candidate, match_all, normalise};
use crate::web::Draft;

use super::{Imported, ImportedLine, ImportedStep};

/// Reads a draft without AI.
///
/// Each line is parsed and matched to a catalogue ingredient by its name, and each step is linked to
/// the lines whose names it mentions as whole words. A line that does not parse counts as unmatched,
/// by its text.
#[must_use]
pub fn from_draft(draft: &Draft, candidates: &[Candidate<'_>]) -> Imported {
    let parsed: Vec<ParsedLine> = draft.lines.iter().map(|line| parse_line(line)).collect();
    let names: Vec<&str> = parsed
        .iter()
        .filter(|line| line.parsed)
        .map(|line| line.name.as_str())
        .collect();
    let matches = match_all(&names, candidates);
    let mut found = matches.linked.into_iter();
    let lines: Vec<ImportedLine> = parsed
        .iter()
        .map(|line| ImportedLine {
            amount: line.amount,
            unit: line.unit,
            name: line.name.clone(),
            note: line.note.clone(),
            ingredient_id: if line.parsed {
                found.next().flatten()
            } else {
                None
            },
            original: line.original.clone(),
        })
        .collect();
    let mut unmatched = matches.unmatched;
    for line in parsed.iter().filter(|line| !line.parsed) {
        push_unique(&mut unmatched, &line.original);
    }
    let steps = draft
        .steps
        .iter()
        .map(|text| ImportedStep {
            text: text.clone(),
            line_positions: positions_named_in(text, &parsed),
        })
        .collect();
    Imported {
        title: draft.title.clone(),
        servings: draft.servings,
        image: draft.image.clone(),
        photo: None,
        categories: Vec::new(),
        lines,
        steps,
        unmatched,
        ai_error: None,
    }
}

fn positions_named_in(step: &str, parsed: &[ParsedLine]) -> Vec<usize> {
    let text = normalise(step);
    parsed
        .iter()
        .enumerate()
        .filter(|(_, line)| line.parsed)
        .filter(|(_, line)| {
            let name = normalise(&line.name);
            !name.is_empty() && whole_word_in(&text, &name)
        })
        .map(|(position, _)| position)
        .collect()
}

fn whole_word_in(text: &str, word: &str) -> bool {
    text.match_indices(word).any(|(start, _)| {
        let end = start + word.len();
        let before = text.get(..start).and_then(|head| head.chars().next_back());
        let after = text.get(end..).and_then(|tail| tail.chars().next());
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    })
}

fn push_unique(names: &mut Vec<String>, name: &str) {
    let key = normalise(name);
    if !names.iter().any(|seen| normalise(seen) == key) {
        names.push(name.to_owned());
    }
}
