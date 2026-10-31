use crabbage_domain::quantity::Unit;
use crabbage_import::pipeline::from_draft;

use crate::{candidate, draft};

#[test]
fn lines_match_by_name_and_unmatched_or_unparsed_lines_are_listed() {
    let flour = candidate(1, "flour");
    let result = from_draft(
        &draft(&["200 g flour", "2 eggs", "***"], &["Mix."]),
        &[flour],
    );
    assert_eq!(result.lines[0].ingredient_id, Some(1));
    assert_eq!(result.lines[0].amount, Some(200.0));
    assert_eq!(result.lines[0].unit, Some(Unit::Gram));
    assert_eq!(result.lines[1].ingredient_id, None);
    assert_eq!(result.lines[1].name, "eggs");
    assert_eq!(result.lines[2].ingredient_id, None);
    assert_eq!(result.lines[2].original, "***");
    assert_eq!(result.unmatched, vec!["eggs".to_owned(), "***".to_owned()]);
    assert_eq!(result.ai_error, None);
}

#[test]
fn a_matched_ingredient_is_found_by_alias_and_unmatched_names_are_once() {
    let aliases = vec!["Plain flour".to_owned()];
    let flour = crabbage_import::matching::Candidate {
        id: 4,
        name: "flour",
        aliases: &aliases,
    };
    let result = from_draft(
        &draft(&["100 g plain flour", "1 egg", "1 Egg"], &[]),
        &[flour],
    );
    assert_eq!(result.lines[0].ingredient_id, Some(4));
    assert_eq!(result.unmatched, vec!["egg".to_owned()]);
}

#[test]
fn a_draft_keeps_its_title_servings_and_photo() {
    let result = from_draft(&draft(&["1 egg"], &["Cook."]), &[]);
    assert_eq!(result.title, "Lemon Pancakes");
    assert_eq!(result.servings, Some(2));
    assert_eq!(
        result.image.as_deref(),
        Some("https://example.com/pancakes.jpg")
    );
}

#[test]
fn steps_link_to_the_lines_they_name_as_whole_words() {
    let result = from_draft(
        &draft(
            &["1 tsp salt", "1 cup unsalted butter", "2 eggs"],
            &[
                "Add the salt.",
                "Melt the unsalted butter.",
                "Beat the eggs and salt.",
            ],
        ),
        &[],
    );
    assert_eq!(result.steps[0].line_positions, vec![0]);
    assert_eq!(result.steps[1].line_positions, vec![1]);
    assert_eq!(result.steps[2].line_positions, vec![0, 2]);
}

#[test]
fn dutch_steps_link_by_whole_word_too() {
    let result = from_draft(
        &draft(
            &["1 tl zout", "100 g ongezouten boter", "2 eieren"],
            &[
                "Voeg het zout toe.",
                "Smelt de ongezouten boter.",
                "Klop de eieren en het eieren.",
            ],
        ),
        &[],
    );
    assert_eq!(result.steps[0].line_positions, vec![0]);
    assert_eq!(result.steps[1].line_positions, vec![1]);
    assert_eq!(result.steps[2].line_positions, vec![2]);
}

#[test]
fn a_line_with_no_name_links_to_no_step() {
    let result = from_draft(&draft(&["***"], &["Stir *** well."]), &[]);
    assert_eq!(result.steps[0].line_positions, Vec::<usize>::new());
}

#[test]
fn a_line_named_in_a_step_twice_is_linked_once() {
    let result = from_draft(
        &draft(&["1 onion"], &["Chop the onion, then the onion."]),
        &[],
    );
    assert_eq!(result.steps[0].line_positions, vec![0]);
}
