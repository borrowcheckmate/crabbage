//! The readable text of a recipe line, in the words and decimal separator of the language.

use crabbage_domain::quantity::{Dimension, Unit};

use super::types::LineRow;
use super::units::unit_text;
use crate::i18n::Messages;

/// The text of `line` in `messages`: `2 Egg`, `250 g Flour, sifted` or `Salt, to taste`.
///
/// A count shows no unit word. A unitless unit follows the ingredient after a comma and replaces
/// the amount. A unit that is not known shows as it is typed.
pub(crate) fn line_text(line: &LineRow, messages: &Messages) -> String {
    let (name, quantity) = ingredient_parts(line, messages);
    join(&[&quantity, &name], " ")
}

/// The ingredient name and quantity as separate page labels.
pub(crate) fn ingredient_parts(line: &LineRow, messages: &Messages) -> (String, String) {
    let symbol = line.unit.as_deref().unwrap_or_default();
    let ingredient = line.ingredient.as_deref().unwrap_or_default();
    let unit = Unit::from_symbol(symbol);
    let note = line.note.as_deref().unwrap_or_default();
    if unit.is_some_and(|unit| unit.dimension() == Dimension::Unitless) {
        return (
            join(&[ingredient, &unit_text(messages, symbol), note], ", "),
            String::new(),
        );
    }
    let amount = line
        .amount
        .filter(|amount| amount.is_finite())
        .map(|amount| number_text(amount, messages.get("number.decimal")))
        .unwrap_or_default();
    let word = if unit == Some(Unit::Piece) {
        String::new()
    } else {
        unit_text(messages, symbol)
    };
    (
        join(&[ingredient, note], ", "),
        join(&[&amount, &word], " "),
    )
}

/// `amount` rounded to two decimals, without trailing zeros, with `decimal` for the point.
fn number_text(amount: f64, decimal: &str) -> String {
    format!("{amount:.2}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .replace('.', decimal)
}

/// The non-empty parts joined with `separator`.
fn join(parts: &[&str], separator: &str) -> String {
    parts
        .iter()
        .copied()
        .filter(|part| !part.is_empty())
        .collect::<Vec<&str>>()
        .join(separator)
}

#[cfg(test)]
mod tests {
    use super::{ingredient_parts, line_text};
    use crate::i18n::{Language, Messages};
    use crate::recipes::types::LineRow;

    fn line(
        amount: Option<f64>,
        unit: Option<&str>,
        ingredient: &str,
        note: Option<&str>,
    ) -> LineRow {
        LineRow {
            amount,
            unit: unit.map(str::to_owned),
            ingredient: Some(ingredient.to_owned()),
            note: note.map(str::to_owned),
        }
    }

    fn en(line: &LineRow) -> String {
        line_text(line, Messages::for_language(Language::En))
    }

    fn nl(line: &LineRow) -> String {
        line_text(line, Messages::for_language(Language::Nl))
    }

    #[test]
    fn a_count_shows_no_unit_word() {
        assert_eq!(en(&line(Some(2.0), Some("piece"), "Egg", None)), "2 Egg");
        assert_eq!(nl(&line(Some(2.0), Some("piece"), "Ei", None)), "2 Ei");
    }

    #[test]
    fn a_unitless_amount_follows_the_ingredient() {
        assert_eq!(
            en(&line(None, Some("to taste"), "Salt", None)),
            "Salt, to taste"
        );
        assert_eq!(
            en(&line(None, Some("pinch"), "Salt", None)),
            "Salt, a pinch"
        );
        assert_eq!(
            nl(&line(None, Some("to taste"), "Zout", None)),
            "Zout, naar smaak"
        );
        assert_eq!(
            nl(&line(None, Some("pinch"), "Zout", None)),
            "Zout, een snufje"
        );
    }

    #[test]
    fn a_measured_amount_shows_its_unit_word_before_the_ingredient() {
        assert_eq!(
            en(&line(Some(250.0), Some("g"), "Flour", None)),
            "250 g Flour"
        );
        assert_eq!(
            nl(&line(Some(1.0), Some("tbsp"), "Suiker", None)),
            "1 el Suiker"
        );
        assert_eq!(nl(&line(Some(1.0), Some("tsp"), "Zout", None)), "1 tl Zout");
    }

    #[test]
    fn decimals_use_the_language_separator_and_drop_trailing_zeros() {
        assert_eq!(
            en(&line(Some(0.5), Some("tsp"), "Vanilla", None)),
            "0.5 tsp Vanilla"
        );
        assert_eq!(
            nl(&line(Some(0.5), Some("tsp"), "Vanilla", None)),
            "0,5 tl Vanilla"
        );
        assert_eq!(en(&line(Some(2.0), Some("g"), "Flour", None)), "2 g Flour");
        assert_eq!(
            en(&line(Some(1.25), Some("g"), "Flour", None)),
            "1.25 g Flour"
        );
        assert_eq!(
            nl(&line(Some(1.25), Some("g"), "Flour", None)),
            "1,25 g Flour"
        );
        assert_eq!(
            en(&line(Some(10.0), Some("g"), "Flour", None)),
            "10 g Flour"
        );
    }

    #[test]
    fn a_line_without_amount_or_unit_is_the_ingredient() {
        assert_eq!(en(&line(None, None, "Flour", None)), "Flour");
    }

    #[test]
    fn the_note_follows_after_a_comma() {
        assert_eq!(
            en(&line(Some(250.0), Some("g"), "Flour", Some("sifted"))),
            "250 g Flour, sifted"
        );
        assert_eq!(
            en(&line(None, Some("to taste"), "Salt", Some("fine"))),
            "Salt, to taste, fine"
        );
    }

    #[test]
    fn ingredient_names_and_quantities_are_separate_page_labels() {
        assert_eq!(
            ingredient_parts(
                &line(Some(250.0), Some("g"), "Flour", Some("sifted")),
                Messages::for_language(Language::En)
            ),
            ("Flour, sifted".to_owned(), "250 g".to_owned())
        );
        assert_eq!(
            ingredient_parts(
                &line(Some(2.0), Some("piece"), "Egg", None),
                Messages::for_language(Language::En)
            ),
            ("Egg".to_owned(), "2".to_owned())
        );
        assert_eq!(
            ingredient_parts(
                &line(None, Some("to taste"), "Salt", None),
                Messages::for_language(Language::En)
            ),
            ("Salt, to taste".to_owned(), String::new())
        );
    }

    #[test]
    fn an_unknown_unit_shows_as_typed() {
        assert_eq!(
            en(&line(Some(2.0), Some("cup"), "Rice", None)),
            "2 cup Rice"
        );
    }
}
