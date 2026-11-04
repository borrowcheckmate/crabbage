//! Unit symbols: every unit is found by its symbol, and other text is not a unit.

use crabbage_domain::quantity::Unit;

#[test]
fn every_unit_is_found_by_its_symbol() {
    for unit in Unit::ALL {
        assert_eq!(Unit::from_symbol(unit.symbol()), Some(unit));
    }
}

#[test]
fn other_text_is_not_a_unit() {
    assert_eq!(Unit::from_symbol(""), None);
    assert_eq!(Unit::from_symbol("cup"), None);
    assert_eq!(Unit::from_symbol("KG"), None);
}
