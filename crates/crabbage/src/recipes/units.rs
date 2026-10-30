//! The unit words the recipe pages show: each unit's symbol or word from the locale files. A symbol
//! that is not a unit shows as it is.

use crabbage_domain::quantity::Unit;

use crate::i18n::Messages;

/// The text a unit symbol shows as in `messages`. Unknown symbols show as they are.
pub(crate) fn unit_text(messages: &Messages, symbol: &str) -> String {
    Unit::from_symbol(symbol).map_or_else(
        || symbol.to_owned(),
        |unit| messages.get(message_key(unit)).to_owned(),
    )
}

const fn message_key(unit: Unit) -> &'static str {
    match unit {
        Unit::Gram => "unit.g",
        Unit::Kilogram => "unit.kg",
        Unit::Millilitre => "unit.ml",
        Unit::Litre => "unit.l",
        Unit::Teaspoon => "unit.tsp",
        Unit::Tablespoon => "unit.tbsp",
        Unit::Piece => "unit.piece",
        Unit::Pinch => "unit.pinch",
        Unit::ToTaste => "unit.to_taste",
    }
}
