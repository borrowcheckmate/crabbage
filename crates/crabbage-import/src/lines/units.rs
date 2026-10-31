//! Units written in English and Dutch, the quantity each one means, and the words that follow.

use crabbage_domain::quantity::{ImperialUnit, Quantity, Unit};

use super::words;

/// What a written unit measures, and how an amount in it becomes a quantity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Measure {
    /// A domain unit, kept as written.
    Plain(Unit),
    /// A domain unit reached by a factor from the written unit: 1 dl is 100 ml.
    Scaled(Unit, f64),
    /// An imperial unit, converted to metric.
    Imperial(ImperialUnit),
}

impl Measure {
    pub(crate) fn quantity(self, amount: f64) -> Option<Quantity> {
        match self {
            Self::Plain(unit) => Quantity::new(amount, unit),
            Self::Scaled(unit, factor) => Quantity::new(amount * factor, unit),
            Self::Imperial(unit) => unit.to_metric(amount),
        }
    }
}

const MEASURES: &[(&str, Measure)] = &[
    ("g", Measure::Plain(Unit::Gram)),
    ("gr", Measure::Plain(Unit::Gram)),
    ("gram", Measure::Plain(Unit::Gram)),
    ("grams", Measure::Plain(Unit::Gram)),
    ("kg", Measure::Plain(Unit::Kilogram)),
    ("kilo", Measure::Plain(Unit::Kilogram)),
    ("kilogram", Measure::Plain(Unit::Kilogram)),
    ("kilograms", Measure::Plain(Unit::Kilogram)),
    ("ml", Measure::Plain(Unit::Millilitre)),
    ("millilitre", Measure::Plain(Unit::Millilitre)),
    ("millilitres", Measure::Plain(Unit::Millilitre)),
    ("milliliter", Measure::Plain(Unit::Millilitre)),
    ("milliliters", Measure::Plain(Unit::Millilitre)),
    ("cl", Measure::Scaled(Unit::Millilitre, 10.0)),
    ("dl", Measure::Scaled(Unit::Millilitre, 100.0)),
    ("l", Measure::Plain(Unit::Litre)),
    ("litre", Measure::Plain(Unit::Litre)),
    ("litres", Measure::Plain(Unit::Litre)),
    ("liter", Measure::Plain(Unit::Litre)),
    ("liters", Measure::Plain(Unit::Litre)),
    ("tl", Measure::Plain(Unit::Teaspoon)),
    ("theelepel", Measure::Plain(Unit::Teaspoon)),
    ("theelepels", Measure::Plain(Unit::Teaspoon)),
    ("tsp", Measure::Plain(Unit::Teaspoon)),
    ("teaspoon", Measure::Plain(Unit::Teaspoon)),
    ("teaspoons", Measure::Plain(Unit::Teaspoon)),
    ("el", Measure::Plain(Unit::Tablespoon)),
    ("eetlepel", Measure::Plain(Unit::Tablespoon)),
    ("eetlepels", Measure::Plain(Unit::Tablespoon)),
    ("tbsp", Measure::Plain(Unit::Tablespoon)),
    ("tablespoon", Measure::Plain(Unit::Tablespoon)),
    ("tablespoons", Measure::Plain(Unit::Tablespoon)),
    ("stuk", Measure::Plain(Unit::Piece)),
    ("stuks", Measure::Plain(Unit::Piece)),
    ("piece", Measure::Plain(Unit::Piece)),
    ("pieces", Measure::Plain(Unit::Piece)),
    ("pcs", Measure::Plain(Unit::Piece)),
    ("snufje", Measure::Plain(Unit::Pinch)),
    ("snufjes", Measure::Plain(Unit::Pinch)),
    ("pinch", Measure::Plain(Unit::Pinch)),
    ("pinches", Measure::Plain(Unit::Pinch)),
    ("cup", Measure::Imperial(ImperialUnit::Cup)),
    ("cups", Measure::Imperial(ImperialUnit::Cup)),
    ("oz", Measure::Imperial(ImperialUnit::Ounce)),
    ("ounce", Measure::Imperial(ImperialUnit::Ounce)),
    ("ounces", Measure::Imperial(ImperialUnit::Ounce)),
    ("fl oz", Measure::Imperial(ImperialUnit::FluidOunce)),
    ("lb", Measure::Imperial(ImperialUnit::Pound)),
    ("lbs", Measure::Imperial(ImperialUnit::Pound)),
    ("pound", Measure::Imperial(ImperialUnit::Pound)),
    ("pounds", Measure::Imperial(ImperialUnit::Pound)),
];

/// The unit at the start of `text` and the text after it. A `.` after the unit and a following
/// `of` or `of the` are left out.
pub(crate) fn leading(text: &str) -> Option<(Measure, &str)> {
    let (measure, rest) = MEASURES
        .iter()
        .find_map(|(word, measure)| Some((*measure, words::strip(text, word)?)))?;
    let rest = rest.strip_prefix('.').unwrap_or(rest).trim_start();
    Some((measure, drop_of(rest)))
}

fn drop_of(text: &str) -> &str {
    words::strip(text, "of").map_or(text, |after| {
        let after = after.trim_start();
        words::strip(after, "the").unwrap_or(after).trim_start()
    })
}
