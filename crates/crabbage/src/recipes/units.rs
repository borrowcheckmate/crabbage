//! The unit words the recipe pages show: the symbol of each unit, translated for the three that
//! have no symbol.

use crabbage_domain::quantity::Unit;

use crate::i18n::t;

/// The text a unit symbol shows as. Unknown symbols show as they are.
pub(crate) fn unit_text(symbol: &str) -> String {
    match Unit::from_symbol(symbol) {
        Some(Unit::Piece) => t("unit.piece").to_owned(),
        Some(Unit::Pinch) => t("unit.pinch").to_owned(),
        Some(Unit::ToTaste) => t("unit.to_taste").to_owned(),
        Some(unit) => unit.symbol().to_owned(),
        None => symbol.to_owned(),
    }
}
