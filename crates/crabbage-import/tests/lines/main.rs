//! Ingredient lines: amounts, units, names and notes in English and Dutch, and lines that stay
//! unparsed.

mod table;

use crabbage_domain::quantity::Unit;
use crabbage_import::lines::parse_line;
use table::{LINES, UNPARSED};

fn same(actual: Option<f64>, expected: Option<f64>) -> bool {
    match (actual, expected) {
        (Some(actual), Some(expected)) => (actual - expected).abs() < 1e-3,
        (None, None) => true,
        _ => false,
    }
}

#[test]
fn lines_read_amount_unit_name_and_note() {
    for (input, amount, amount_max, unit, name, note) in LINES.iter().copied() {
        let line = parse_line(input);
        assert!(line.parsed, "{input}: {line:?}");
        assert!(same(line.amount, amount), "{input}: amount {line:?}");
        assert!(
            same(line.amount_max, amount_max),
            "{input}: amount_max {line:?}"
        );
        assert_eq!(line.unit, unit, "{input}");
        assert_eq!(line.name, name, "{input}");
        assert_eq!(line.note.as_deref(), note, "{input}");
        assert_eq!(line.original, input);
    }
}

#[test]
fn unparseable_lines_keep_their_text_and_are_flagged() {
    for input in UNPARSED {
        let line = parse_line(input);
        assert!(!line.parsed, "{input:?}: {line:?}");
        assert_eq!(line.amount, None, "{input:?}");
        assert_eq!(line.amount_max, None, "{input:?}");
        assert_eq!(line.unit, None, "{input:?}");
        assert_eq!(line.note, None, "{input:?}");
        assert_eq!(line.original, input);
        assert_eq!(
            line.name,
            input.split_whitespace().collect::<Vec<_>>().join(" ")
        );
    }
}

#[test]
fn a_comma_or_parenthesis_starts_the_note() {
    let line = parse_line("1 onion, diced (large)");
    assert_eq!(line.name, "onion");
    assert_eq!(line.note.as_deref(), Some("diced (large)"));
}

#[test]
fn an_empty_note_is_none() {
    let line = parse_line("2 onions, ");
    assert_eq!(line.name, "onions");
    assert_eq!(line.note, None);
}

#[test]
fn a_range_keeps_the_unit_of_its_lower_end() {
    let line = parse_line("1-2 cups rice");
    assert_eq!(line.unit, Some(Unit::Millilitre));
    assert!(same(line.amount, Some(240.0)));
    assert!(same(line.amount_max, Some(480.0)));
}

#[test]
fn a_parenthesised_note_keeps_the_text_after_it() {
    assert_eq!(
        parse_line("200 g butter (soft), melted").note.as_deref(),
        Some("soft, melted")
    );
    assert_eq!(
        parse_line("200 g butter (soft").note.as_deref(),
        Some("soft")
    );
}

#[test]
fn to_taste_without_an_amount_becomes_the_unit() {
    for (input, name) in [
        ("Salt to taste", "Salt"),
        ("zout naar smaak", "zout"),
        ("Pepper, to taste", "Pepper"),
        ("Peper (naar smaak)", "Peper"),
    ] {
        let line = parse_line(input);
        assert_eq!(line.unit, Some(Unit::ToTaste), "{input}");
        assert_eq!(line.name, name, "{input}");
        assert_eq!(line.note, None, "{input}");
    }
    let alone = parse_line("to taste");
    assert_eq!((alone.unit, alone.name.as_str()), (None, "to taste"));
    assert_eq!(
        parse_line("2 g salt, to taste").note.as_deref(),
        Some("to taste")
    );
}
