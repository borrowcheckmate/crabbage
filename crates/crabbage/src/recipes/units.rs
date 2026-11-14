//! The unit labels shown to people, in the current language.

use crabbage_domain::quantity::Unit;

use crate::i18n::Messages;

/// The text a unit symbol shows as in `messages`, in the plural form when `plural` is set.
/// Unknown symbols show as they are.
pub(crate) fn unit_text(messages: &Messages, symbol: &str, plural: bool) -> String {
    Unit::from_symbol(symbol).map_or_else(
        || symbol.to_owned(),
        |unit| messages.get(message_key(unit, plural)).to_owned(),
    )
}

/// Whether an amount reads in the plural: every amount except one.
pub(crate) fn is_plural(amount: f64) -> bool {
    (amount - 1.0).abs() > f64::EPSILON
}

pub(crate) fn unit_option_text(messages: &Messages, unit: Unit) -> String {
    if unit == Unit::Piece {
        messages.get("unit.piece_option").to_owned()
    } else {
        messages.get(message_key(unit, true)).to_owned()
    }
}

const fn message_key(unit: Unit, plural: bool) -> &'static str {
    match unit {
        Unit::Gram => "unit.g",
        Unit::Kilogram => "unit.kg",
        Unit::Millilitre => "unit.ml",
        Unit::Litre => "unit.l",
        Unit::Teaspoon => "unit.tsp",
        Unit::Tablespoon => "unit.tbsp",
        Unit::Piece => "unit.piece_label",
        Unit::Slice if plural => "unit.slices",
        Unit::Slice => "unit.slice",
        Unit::Clove if plural => "unit.cloves",
        Unit::Clove => "unit.clove",
        Unit::Can if plural => "unit.cans",
        Unit::Can => "unit.can",
        Unit::Pack if plural => "unit.packs",
        Unit::Pack => "unit.pack",
        Unit::Bunch if plural => "unit.bunches",
        Unit::Bunch => "unit.bunch",
        Unit::Pinch => "unit.pinch",
        Unit::ToTaste => "unit.to_taste",
    }
}
