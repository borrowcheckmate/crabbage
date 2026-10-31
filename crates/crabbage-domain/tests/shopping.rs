//! Shopping list merging: ingredient lines, amounts that add or stay apart, text items and order.
#![expect(
    clippy::unwrap_used,
    reason = "helpers build quantities from literals known to be valid"
)]

use crabbage_domain::quantity::{Quantity, Unit};
use crabbage_domain::shopping::{Item, Line, merge};

const EPSILON: f64 = 1e-9;

fn quantity(amount: f64, unit: Unit) -> Quantity {
    Quantity::new(amount, unit).unwrap()
}

const fn ingredient(id: i64) -> Item {
    Item::Ingredient(id)
}

fn text(value: &str) -> Item {
    Item::Text(value.to_owned())
}

fn assert_amount(actual: Quantity, amount: f64, unit: Unit) {
    assert!(
        (actual.amount() - amount).abs() < EPSILON,
        "{} != {amount}",
        actual.amount()
    );
    assert_eq!(actual.unit(), unit);
}

#[test]
fn same_ingredient_merges_into_one_line_with_the_best_unit() {
    let lines = merge([
        (ingredient(1), Some(quantity(600.0, Unit::Gram))),
        (ingredient(1), Some(quantity(500.0, Unit::Gram))),
    ]);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].item, ingredient(1));
    assert_eq!(lines[0].amounts.len(), 1);
    assert_amount(lines[0].amounts[0], 1.1, Unit::Kilogram);
}

#[test]
fn amounts_in_different_dimensions_stay_apart_in_order() {
    let lines = merge([
        (ingredient(2), Some(quantity(100.0, Unit::Gram))),
        (ingredient(2), Some(quantity(2.0, Unit::Tablespoon))),
    ]);
    assert_eq!(lines.len(), 1);
    let amounts = &lines[0].amounts;
    assert_eq!(amounts.len(), 2);
    assert_amount(amounts[0], 100.0, Unit::Gram);
    assert_amount(amounts[1], 2.0, Unit::Tablespoon);
}

#[test]
fn mixed_spoons_add_in_the_base_unit() {
    let lines = merge([
        (ingredient(3), Some(quantity(1.0, Unit::Teaspoon))),
        (ingredient(3), Some(quantity(1.0, Unit::Tablespoon))),
    ]);
    assert_eq!(lines[0].amounts.len(), 1);
    assert_amount(lines[0].amounts[0], 20.0, Unit::Millilitre);
}

#[test]
fn text_items_never_merge() {
    let lines = merge([
        (text("butter"), None),
        (ingredient(1), None),
        (text("butter"), None),
    ]);
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].item, text("butter"));
    assert_eq!(lines[1].item, ingredient(1));
    assert_eq!(lines[2].item, text("butter"));
}

#[test]
fn lines_keep_first_seen_order() {
    let lines = merge([
        (ingredient(5), Some(quantity(1.0, Unit::Piece))),
        (ingredient(4), Some(quantity(1.0, Unit::Piece))),
        (ingredient(5), Some(quantity(2.0, Unit::Piece))),
    ]);
    let items: Vec<Item> = lines.iter().map(|line| line.item.clone()).collect();
    assert_eq!(items, [ingredient(5), ingredient(4)]);
}

#[test]
fn an_item_without_amount_merges_without_adding_one() {
    let lines = merge([
        (ingredient(6), Some(quantity(250.0, Unit::Millilitre))),
        (ingredient(6), None),
    ]);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].amounts.len(), 1);
    assert_amount(lines[0].amounts[0], 250.0, Unit::Millilitre);

    let lines = merge([(ingredient(7), None), (ingredient(7), None)]);
    assert_eq!(
        lines,
        [Line {
            item: ingredient(7),
            amounts: Vec::new(),
        }]
    );
}

#[test]
fn unitless_amounts_stay_apart_unless_equal() {
    let lines = merge([
        (ingredient(8), Some(quantity(1.0, Unit::Pinch))),
        (ingredient(8), Some(quantity(1.0, Unit::ToTaste))),
    ]);
    assert_eq!(lines[0].amounts.len(), 2);
    assert_amount(lines[0].amounts[0], 1.0, Unit::Pinch);
    assert_amount(lines[0].amounts[1], 1.0, Unit::ToTaste);

    let lines = merge([
        (ingredient(9), Some(quantity(1.0, Unit::Pinch))),
        (ingredient(9), Some(quantity(1.0, Unit::Pinch))),
    ]);
    assert_eq!(lines[0].amounts.len(), 1);
    assert_amount(lines[0].amounts[0], 2.0, Unit::Pinch);
}

#[test]
fn no_items_make_no_lines() {
    assert_eq!(merge(std::iter::empty()), Vec::new());
}
