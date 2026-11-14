//! The draft, candidates and complete recipe the pipeline tests share.

use crabbage_import::matching::Candidate;
use crabbage_import::pipeline::{Imported, ImportedLine, ImportedStep};
use crabbage_import::web::Draft;

pub(crate) fn draft(lines: &[&str], steps: &[&str]) -> Draft {
    Draft {
        title: "Lemon Pancakes".to_owned(),
        servings: Some(2),
        language: Some("en".to_owned()),
        image: Some("https://example.com/pancakes.jpg".to_owned()),
        lines: lines.iter().map(|line| (*line).to_owned()).collect(),
        steps: steps.iter().map(|step| (*step).to_owned()).collect(),
        prep_time: None,
        cook_time: None,
        total_time: None,
    }
}

pub(crate) const fn candidate(id: i64, name: &str) -> Candidate<'_> {
    Candidate {
        id,
        name,
        aliases: &[],
    }
}

pub(crate) fn complete_recipe() -> Imported {
    Imported {
        title: "Lemon Pancakes".to_owned(),
        servings: Some(2),
        image: None,
        photo: None,
        categories: Vec::new(),
        lines: vec![ImportedLine {
            amount: Some(200.0),
            unit: None,
            name: "flour".to_owned(),
            note: None,
            ingredient_id: Some(1),
            section_id: None,
            original: "200 flour".to_owned(),
        }],
        steps: vec![ImportedStep {
            text: "Mix.".to_owned(),
            line_positions: vec![0],
        }],
        unmatched: Vec::new(),
        ai_error: None,
    }
}
