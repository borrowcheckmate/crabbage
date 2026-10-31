use crabbage_import::pipeline::{Imported, needs_ai};

use crate::complete_recipe;

fn complete() -> Imported {
    complete_recipe()
}

#[test]
fn a_complete_draft_in_the_instance_language_needs_no_ai() {
    assert!(!needs_ai(&complete(), Some("en"), "en"));
}

#[test]
fn a_blank_title_needs_ai() {
    let recipe = Imported {
        title: "  ".to_owned(),
        ..complete()
    };
    assert!(needs_ai(&recipe, Some("en"), "en"));
}

#[test]
fn missing_servings_need_ai() {
    let recipe = Imported {
        servings: None,
        ..complete()
    };
    assert!(needs_ai(&recipe, Some("en"), "en"));
}

#[test]
fn no_lines_need_ai() {
    let recipe = Imported {
        lines: Vec::new(),
        ..complete()
    };
    assert!(needs_ai(&recipe, Some("en"), "en"));
}

#[test]
fn no_steps_need_ai() {
    let recipe = Imported {
        steps: Vec::new(),
        ..complete()
    };
    assert!(needs_ai(&recipe, Some("en"), "en"));
}

#[test]
fn an_unmatched_line_needs_ai() {
    let recipe = Imported {
        unmatched: vec!["eggs".to_owned()],
        ..complete()
    };
    assert!(needs_ai(&recipe, Some("en"), "en"));
}

#[test]
fn a_page_in_another_language_needs_ai() {
    assert!(needs_ai(&complete(), Some("nl"), "en"));
}

#[test]
fn a_regional_tag_of_the_instance_language_needs_no_ai() {
    assert!(!needs_ai(&complete(), Some("nl-NL"), "nl"));
    assert!(!needs_ai(&complete(), Some("nl_BE"), "NL"));
}

#[test]
fn a_different_primary_language_needs_ai_even_with_a_region() {
    assert!(needs_ai(&complete(), Some("en-GB"), "nl"));
}

#[test]
fn an_unknown_page_language_alone_does_not_need_ai() {
    assert!(!needs_ai(&complete(), None, "en"));
}
