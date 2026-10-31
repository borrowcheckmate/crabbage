use crabbage_domain::quantity::Unit;
use crabbage_import::ai::{AiDraft, AiLine, AiStep};
use crabbage_import::pipeline::{from_draft, merge};

use crate::{candidate, complete_recipe, draft};

fn ai_line(amount: Option<f64>, unit: Option<&str>, name: &str) -> AiLine {
    AiLine {
        amount,
        unit: unit.map(str::to_owned),
        name: name.to_owned(),
        note: None,
        ingredient_id: None,
    }
}

fn ai_draft(lines: Vec<AiLine>, steps: Vec<AiStep>) -> AiDraft {
    AiDraft {
        title: "Fluffy pancakes".to_owned(),
        servings: None,
        lines,
        steps,
        prep_minutes: None,
        cook_minutes: None,
    }
}

fn step(text: &str, lines: Vec<usize>) -> AiStep {
    AiStep {
        text: text.to_owned(),
        lines,
    }
}

#[test]
fn imperial_units_become_metric() {
    let ai = ai_draft(
        vec![
            ai_line(Some(1.0), Some("cup"), "milk"),
            ai_line(Some(2.0), Some("cups"), "water"),
            ai_line(Some(4.0), Some("oz"), "butter"),
            ai_line(Some(1.5), Some("dl"), "cream"),
        ],
        vec![step("Mix.", vec![])],
    );
    let result = merge(&ai, &complete_recipe(), &[]);
    assert_eq!(result.lines[0].unit, Some(Unit::Millilitre));
    assert_eq!(result.lines[0].amount, Some(240.0));
    assert_eq!(result.lines[1].amount, Some(480.0));
    assert_eq!(result.lines[2].unit, Some(Unit::Gram));
    assert!((result.lines[2].amount.unwrap_or_default() - 113.398).abs() < 0.001);
    assert_eq!(result.lines[3].unit, Some(Unit::Millilitre));
    assert_eq!(result.lines[3].amount, Some(150.0));
}

#[test]
fn a_measure_without_an_amount_has_no_unit() {
    let ai = ai_draft(
        vec![ai_line(None, Some("cup"), "flour")],
        vec![step("Mix.", vec![])],
    );
    let result = merge(&ai, &complete_recipe(), &[]);
    assert_eq!(result.lines[0].amount, None);
    assert_eq!(result.lines[0].unit, None);
    assert_eq!(result.lines[0].name, "flour");
}

#[test]
fn domain_units_and_amounts_are_kept() {
    let ai = ai_draft(
        vec![
            ai_line(Some(200.0), Some("g"), "flour"),
            ai_line(Some(2.0), Some("tbsp"), "oil"),
            ai_line(None, Some("to taste"), "salt"),
        ],
        vec![step("Mix.", vec![])],
    );
    let result = merge(&ai, &complete_recipe(), &[]);
    assert_eq!(result.lines[0].amount, Some(200.0));
    assert_eq!(result.lines[0].unit, Some(Unit::Gram));
    assert_eq!(result.lines[1].unit, Some(Unit::Tablespoon));
    assert_eq!(result.lines[1].amount, Some(2.0));
    assert_eq!(result.lines[2].unit, Some(Unit::ToTaste));
    assert_eq!(result.lines[2].original, "salt to taste");
}

#[test]
fn a_bogus_ingredient_id_is_dropped_and_matched_by_name() {
    let flour = candidate(1, "flour");
    let mut line = ai_line(Some(200.0), Some("g"), "Flour");
    line.ingredient_id = Some(99);
    let ai = ai_draft(vec![line], vec![step("Mix.", vec![])]);
    let result = merge(&ai, &complete_recipe(), &[flour]);
    assert_eq!(result.lines[0].ingredient_id, Some(1));
}

#[test]
fn a_valid_ingredient_id_is_kept_even_when_the_name_differs() {
    let milk = candidate(2, "milk");
    let mut line = ai_line(Some(1.0), Some("cup"), "flour");
    line.ingredient_id = Some(2);
    let ai = ai_draft(vec![line], vec![step("Mix.", vec![])]);
    let result = merge(&ai, &complete_recipe(), &[milk]);
    assert_eq!(result.lines[0].ingredient_id, Some(2));
}

#[test]
fn steps_take_the_ai_line_positions_sorted_and_without_duplicates() {
    let ai = ai_draft(
        vec![
            ai_line(Some(1.0), Some("piece"), "egg"),
            ai_line(Some(2.0), Some("piece"), "banana"),
        ],
        vec![step("Whisk.", vec![1, 0, 1])],
    );
    let result = merge(&ai, &complete_recipe(), &[]);
    assert_eq!(result.steps[0].text, "Whisk.");
    assert_eq!(result.steps[0].line_positions, vec![0, 1]);
}

#[test]
fn the_title_comes_from_the_ai_and_the_photo_from_the_page() {
    let base = from_draft(&draft(&["1 egg"], &["Cook."]), &[]);
    let ai = ai_draft(
        vec![ai_line(Some(1.0), Some("piece"), "egg")],
        vec![step("Cook.", vec![0])],
    );
    let result = merge(&ai, &base, &[]);
    assert_eq!(result.title, "Fluffy pancakes");
    assert_eq!(result.image, base.image);
}

#[test]
fn servings_fall_back_to_the_page_when_the_ai_has_none() {
    let ai = ai_draft(
        vec![ai_line(Some(1.0), Some("piece"), "egg")],
        vec![step("Cook.", vec![0])],
    );
    assert_eq!(merge(&ai, &complete_recipe(), &[]).servings, Some(2));
    let with_servings = AiDraft {
        servings: Some(6),
        ..ai
    };
    assert_eq!(
        merge(&with_servings, &complete_recipe(), &[]).servings,
        Some(6)
    );
}

#[test]
fn unmatched_names_are_recomputed_from_the_lines_without_an_id() {
    let flour = candidate(1, "flour");
    let mut known = ai_line(Some(200.0), Some("g"), "flour");
    known.ingredient_id = None;
    let ai = ai_draft(
        vec![
            known,
            ai_line(Some(2.0), Some("piece"), "Eggs"),
            ai_line(Some(2.0), Some("piece"), "eggs"),
        ],
        vec![step("Cook.", vec![0])],
    );
    let mut base = complete_recipe();
    base.unmatched = vec!["stale".to_owned()];
    let result = merge(&ai, &base, &[flour]);
    assert_eq!(result.unmatched, vec!["Eggs".to_owned()]);
}

#[test]
fn each_line_keeps_its_text_as_metric_with_its_note() {
    let mut line = ai_line(Some(1.0), Some("cup"), "milk");
    line.note = Some("warm".to_owned());
    let ai = ai_draft(vec![line], vec![step("Heat.", vec![0])]);
    let result = merge(&ai, &complete_recipe(), &[]);
    assert_eq!(result.lines[0].original, "240 ml milk, warm");
    assert_eq!(result.lines[0].note.as_deref(), Some("warm"));
}
