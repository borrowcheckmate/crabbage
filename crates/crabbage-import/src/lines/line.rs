//! One ingredient line split into its amount, unit, name and note.

use crabbage_domain::quantity::Unit;

use super::numbers::{self, Amount};
use super::units::{self, Measure};

/// An ingredient line, read as far as its text allows.
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedLine {
    /// The amount, or the lower end of a range. `None` when the line has no amount.
    pub amount: Option<f64>,
    /// The upper end of a range, such as 3 in `2-3`.
    pub amount_max: Option<f64>,
    /// The unit of the amount. Units without a domain counterpart stay in the name.
    pub unit: Option<Unit>,
    /// The ingredient name.
    pub name: String,
    /// The text after a comma or in parentheses, such as `finely chopped`.
    pub note: Option<String>,
    /// The line as it was given.
    pub original: String,
    /// `false` when the line has no name, see [`parse_line`].
    pub parsed: bool,
}

/// Reads an ingredient line: an optional amount, an optional unit, the name and an optional note.
///
/// Whitespace is collapsed first. A line is `parsed` when its name contains a letter or a digit.
/// An empty line, one with only punctuation, and one with an amount and unit but no name are not.
/// Those keep the collapsed text as `name`, with no amount, unit or note. A line without an
/// amount is parsed with its whole text as the name, except that a trailing `to taste` or
/// `naar smaak`, also as the note, becomes the unit [`Unit::ToTaste`].
#[must_use]
pub fn parse_line(text: &str) -> ParsedLine {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    match parse(&collapsed) {
        Some(parts) if parts.name.chars().any(char::is_alphanumeric) => ParsedLine {
            amount: parts.amount,
            amount_max: parts.amount_max,
            unit: parts.unit,
            name: parts.name,
            note: parts.note,
            original: text.to_owned(),
            parsed: true,
        },
        _ => ParsedLine {
            amount: None,
            amount_max: None,
            unit: None,
            name: collapsed,
            note: None,
            original: text.to_owned(),
            parsed: false,
        },
    }
}

struct Parts {
    amount: Option<f64>,
    amount_max: Option<f64>,
    unit: Option<Unit>,
    name: String,
    note: Option<String>,
}

impl Parts {
    fn new(amount: Option<f64>, amount_max: Option<f64>, unit: Option<Unit>, text: &str) -> Self {
        let end = text.find([',', '(']).unwrap_or(text.len());
        let (name, tail) = text.split_at_checked(end).unwrap_or((text, ""));
        let note = note_of(tail);
        Self {
            amount,
            amount_max,
            unit,
            name: name.trim().to_owned(),
            note: (!note.is_empty()).then_some(note),
        }
    }

    fn with_to_taste(mut self) -> Self {
        if self.amount.is_some() {
            return self;
        }
        if self.note.as_deref().is_some_and(is_to_taste) {
            self.note = None;
            self.unit = Some(Unit::ToTaste);
        } else if let Some(name) = TO_TASTE
            .iter()
            .find_map(|phrase| strip_suffix(&self.name, phrase))
        {
            self.name = name;
            self.unit = Some(Unit::ToTaste);
        }
        if self.unit != Some(Unit::ToTaste) {
            if let Some((name, note)) = OPTIONAL
                .iter()
                .find_map(|phrase| strip_suffix(&self.name, phrase).map(|name| (name, *phrase)))
            {
                self.name = name;
                self.note = Some(join_note(self.note.take(), note));
            } else if let Some((note, optional)) = self.note.as_deref().and_then(|note| {
                OPTIONAL
                    .iter()
                    .find_map(|phrase| strip_suffix(note, phrase).map(|note| (note, *phrase)))
            }) {
                self.note = Some(format!("{note}, {optional}"));
            }
        }
        self
    }
}

fn join_note(existing: Option<String>, note: &str) -> String {
    existing.map_or_else(|| note.to_owned(), |existing| format!("{existing}, {note}"))
}

const TO_TASTE: [&str; 2] = ["to taste", "naar smaak"];
const OPTIONAL: [&str; 5] = [
    "naar keuze",
    "of choice",
    "optioneel",
    "optional",
    "(optioneel)",
];

fn is_to_taste(text: &str) -> bool {
    TO_TASTE
        .iter()
        .any(|phrase| text.eq_ignore_ascii_case(phrase))
}

fn strip_suffix(name: &str, phrase: &str) -> Option<String> {
    let start = name.len().checked_sub(phrase.len())?;
    let (head, tail) = name.split_at_checked(start)?;
    let head = head.strip_suffix(' ')?.trim_end();
    (tail.eq_ignore_ascii_case(phrase) && !head.is_empty()).then(|| head.to_owned())
}

fn note_of(tail: &str) -> String {
    let Some(inside) = tail.strip_prefix('(') else {
        return tail.trim_start_matches(',').trim().to_owned();
    };
    let Some((note, rest)) = inside.split_once(')') else {
        return inside.trim().to_owned();
    };
    let rest = rest.trim().trim_start_matches(',').trim();
    [note.trim(), rest]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
}

fn parse(text: &str) -> Option<Parts> {
    let Some((found, after)) = numbers::leading(text) else {
        return Some(Parts::new(None, None, None, text).with_to_taste());
    };
    let after = after.trim_start();
    let (measure, rest) = match units::leading(after) {
        Some((measure, rest)) => (Some(measure), rest),
        None => (None, after),
    };
    let (amount, amount_max, unit) = measured(found, measure)?;
    Some(Parts::new(Some(amount), amount_max, unit, rest))
}

fn measured(found: Amount, measure: Option<Measure>) -> Option<(f64, Option<f64>, Option<Unit>)> {
    let Some(measure) = measure else {
        return Some((found.low, found.high, None));
    };
    let low = measure.quantity(found.low)?;
    let high = match found.high {
        Some(high) => Some(measure.quantity(high)?.convert_to(low.unit())?.amount()),
        None => None,
    };
    Some((low.amount(), high, Some(low.unit())))
}
