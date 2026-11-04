//! Units, conversion, addition, display and imperial input of quantities.
#![expect(
    clippy::unwrap_used,
    reason = "helpers build quantities from literals known to be valid"
)]

use crabbage_domain::quantity::{Dimension, ImperialUnit, Quantity, Unit};

const EPSILON: f64 = 1e-9;

fn quantity(amount: f64, unit: Unit) -> Quantity {
    Quantity::new(amount, unit).unwrap()
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < EPSILON,
        "{actual} != {expected}"
    );
}

fn assert_is(actual: Quantity, amount: f64, unit: Unit) {
    assert_close(actual.amount(), amount);
    assert_eq!(actual.unit(), unit);
}

fn converted(amount: f64, from: Unit, to: Unit) -> Quantity {
    quantity(amount, from).convert_to(to).unwrap()
}

fn sum(a: Quantity, b: Quantity) -> Quantity {
    a.checked_add(b).unwrap()
}

#[test]
fn units_belong_to_their_dimension() {
    assert_eq!(Unit::Gram.dimension(), Dimension::Mass);
    assert_eq!(Unit::Kilogram.dimension(), Dimension::Mass);
    assert_eq!(Unit::Millilitre.dimension(), Dimension::Volume);
    assert_eq!(Unit::Litre.dimension(), Dimension::Volume);
    assert_eq!(Unit::Teaspoon.dimension(), Dimension::Volume);
    assert_eq!(Unit::Tablespoon.dimension(), Dimension::Volume);
    assert_eq!(Unit::Piece.dimension(), Dimension::Count);
    assert_eq!(Unit::Pinch.dimension(), Dimension::Unitless);
    assert_eq!(Unit::ToTaste.dimension(), Dimension::Unitless);
}

#[test]
fn factors_to_the_base_unit() {
    assert_eq!(Unit::Gram.factor(), Some(1.0));
    assert_eq!(Unit::Kilogram.factor(), Some(1000.0));
    assert_eq!(Unit::Millilitre.factor(), Some(1.0));
    assert_eq!(Unit::Litre.factor(), Some(1000.0));
    assert_eq!(Unit::Teaspoon.factor(), Some(5.0));
    assert_eq!(Unit::Tablespoon.factor(), Some(15.0));
    assert_eq!(Unit::Piece.factor(), Some(1.0));
    assert_eq!(Unit::Pinch.factor(), None);
    assert_eq!(Unit::ToTaste.factor(), None);
}

#[test]
fn symbols() {
    let symbols = [
        Unit::Gram,
        Unit::Kilogram,
        Unit::Millilitre,
        Unit::Litre,
        Unit::Teaspoon,
        Unit::Tablespoon,
        Unit::Piece,
        Unit::Pinch,
        Unit::ToTaste,
    ]
    .map(Unit::symbol);
    assert_eq!(
        symbols,
        [
            "g", "kg", "ml", "l", "tsp", "tbsp", "piece", "pinch", "to taste"
        ]
    );
}

#[test]
fn converts_within_mass() {
    assert_is(
        converted(1.5, Unit::Kilogram, Unit::Gram),
        1500.0,
        Unit::Gram,
    );
    assert_is(
        converted(250.0, Unit::Gram, Unit::Kilogram),
        0.25,
        Unit::Kilogram,
    );
}

#[test]
fn converts_within_volume() {
    assert_is(
        converted(0.25, Unit::Litre, Unit::Millilitre),
        250.0,
        Unit::Millilitre,
    );
    assert_is(
        converted(500.0, Unit::Millilitre, Unit::Litre),
        0.5,
        Unit::Litre,
    );
}

#[test]
fn converts_spoons_to_millilitres() {
    assert_is(
        converted(1.0, Unit::Teaspoon, Unit::Millilitre),
        5.0,
        Unit::Millilitre,
    );
    assert_is(
        converted(2.0, Unit::Tablespoon, Unit::Millilitre),
        30.0,
        Unit::Millilitre,
    );
    assert_is(
        converted(1.0, Unit::Tablespoon, Unit::Teaspoon),
        3.0,
        Unit::Teaspoon,
    );
    assert_is(
        converted(10.0, Unit::Millilitre, Unit::Tablespoon),
        10.0 / 15.0,
        Unit::Tablespoon,
    );
}

#[test]
fn converts_to_the_same_unit_unchanged() {
    assert_is(converted(2.0, Unit::Pinch, Unit::Pinch), 2.0, Unit::Pinch);
    assert_is(converted(3.0, Unit::Piece, Unit::Piece), 3.0, Unit::Piece);
}

#[test]
fn never_converts_across_dimensions() {
    assert!(
        quantity(1.0, Unit::Kilogram)
            .convert_to(Unit::Millilitre)
            .is_none()
    );
    assert!(
        quantity(100.0, Unit::Gram)
            .convert_to(Unit::Tablespoon)
            .is_none()
    );
    assert!(quantity(1.0, Unit::Litre).convert_to(Unit::Piece).is_none());
    assert!(quantity(2.0, Unit::Piece).convert_to(Unit::Gram).is_none());
    assert!(quantity(2.0, Unit::Pinch).convert_to(Unit::Gram).is_none());
    assert!(quantity(2.0, Unit::Gram).convert_to(Unit::Pinch).is_none());
    assert!(
        quantity(2.0, Unit::ToTaste)
            .convert_to(Unit::Piece)
            .is_none()
    );
}

#[test]
fn unitless_units_convert_only_to_themselves() {
    assert!(
        quantity(1.0, Unit::Pinch)
            .convert_to(Unit::ToTaste)
            .is_none()
    );
    assert!(
        quantity(1.0, Unit::ToTaste)
            .convert_to(Unit::Pinch)
            .is_none()
    );
}

#[test]
fn display_picks_grams_or_kilograms() {
    assert_is(quantity(1500.0, Unit::Gram).display(), 1.5, Unit::Kilogram);
    assert_is(quantity(999.0, Unit::Gram).display(), 999.0, Unit::Gram);
    assert_is(quantity(1000.0, Unit::Gram).display(), 1.0, Unit::Kilogram);
    assert_is(quantity(0.5, Unit::Kilogram).display(), 500.0, Unit::Gram);
}

#[test]
fn display_picks_millilitres_or_litres() {
    assert_is(
        quantity(0.25, Unit::Litre).display(),
        250.0,
        Unit::Millilitre,
    );
    assert_is(
        quantity(999.0, Unit::Millilitre).display(),
        999.0,
        Unit::Millilitre,
    );
    assert_is(
        quantity(1000.0, Unit::Millilitre).display(),
        1.0,
        Unit::Litre,
    );
}

#[test]
fn display_keeps_spoons_and_unitless_units() {
    assert_is(
        quantity(2.0, Unit::Tablespoon).display(),
        2.0,
        Unit::Tablespoon,
    );
    assert_is(quantity(1.0, Unit::Teaspoon).display(), 1.0, Unit::Teaspoon);
    assert_is(quantity(3.0, Unit::Piece).display(), 3.0, Unit::Piece);
    assert_is(quantity(1.0, Unit::Pinch).display(), 1.0, Unit::Pinch);
    assert_is(quantity(1.0, Unit::ToTaste).display(), 1.0, Unit::ToTaste);
}

#[test]
fn adds_within_a_dimension_in_the_display_unit() {
    assert_is(
        sum(quantity(1.0, Unit::Kilogram), quantity(500.0, Unit::Gram)),
        1.5,
        Unit::Kilogram,
    );
    assert_is(
        sum(quantity(600.0, Unit::Gram), quantity(600.0, Unit::Gram)),
        1.2,
        Unit::Kilogram,
    );
    assert_is(
        sum(
            quantity(300.0, Unit::Millilitre),
            quantity(0.2, Unit::Litre),
        ),
        500.0,
        Unit::Millilitre,
    );
    assert_is(
        sum(
            quantity(0.5, Unit::Litre),
            quantity(750.0, Unit::Millilitre),
        ),
        1.25,
        Unit::Litre,
    );
    assert_is(
        sum(
            quantity(1.0, Unit::Teaspoon),
            quantity(1.0, Unit::Millilitre),
        ),
        6.0,
        Unit::Millilitre,
    );
    assert_is(
        sum(
            quantity(2.0, Unit::Tablespoon),
            quantity(1.0, Unit::Teaspoon),
        ),
        35.0,
        Unit::Millilitre,
    );
}

#[test]
fn adding_the_same_spoon_keeps_spoons() {
    assert_is(
        sum(quantity(1.0, Unit::Teaspoon), quantity(1.0, Unit::Teaspoon)),
        2.0,
        Unit::Teaspoon,
    );
}

#[test]
fn adds_the_same_unitless_unit() {
    assert_is(
        sum(quantity(1.0, Unit::Pinch), quantity(1.0, Unit::Pinch)),
        2.0,
        Unit::Pinch,
    );
    assert_is(
        sum(quantity(2.0, Unit::Piece), quantity(1.0, Unit::Piece)),
        3.0,
        Unit::Piece,
    );
}

#[test]
fn never_adds_across_dimensions_or_different_unitless_units() {
    let pairs = [
        (quantity(1.0, Unit::Kilogram), quantity(1.0, Unit::Litre)),
        (quantity(1.0, Unit::Piece), quantity(1.0, Unit::Gram)),
        (quantity(1.0, Unit::Pinch), quantity(1.0, Unit::Gram)),
        (quantity(1.0, Unit::Pinch), quantity(1.0, Unit::ToTaste)),
    ];
    for (a, b) in pairs {
        assert!(a.checked_add(b).is_none());
    }
}

#[test]
fn addition_overflow_is_none() {
    assert!(
        quantity(f64::MAX, Unit::Gram)
            .checked_add(quantity(f64::MAX, Unit::Gram))
            .is_none()
    );
}

#[test]
fn imperial_ounces_and_pounds_convert_to_grams() {
    assert_is(
        ImperialUnit::Ounce.to_metric(16.0).unwrap(),
        453.59237,
        Unit::Gram,
    );
    assert_is(
        ImperialUnit::Pound.to_metric(1.0).unwrap(),
        453.59237,
        Unit::Gram,
    );
    assert_is(
        ImperialUnit::Pound.to_metric(3.0).unwrap(),
        1.360_777_11,
        Unit::Kilogram,
    );
}

#[test]
fn imperial_cups_and_fluid_ounces_convert_to_millilitres() {
    assert_is(
        ImperialUnit::Cup.to_metric(1.0).unwrap(),
        240.0,
        Unit::Millilitre,
    );
    assert_is(ImperialUnit::Cup.to_metric(5.0).unwrap(), 1.2, Unit::Litre);
    assert_is(
        ImperialUnit::FluidOunce.to_metric(1.0).unwrap(),
        29.573_529_562_5,
        Unit::Millilitre,
    );
}

#[test]
fn imperial_rejects_negative_and_non_finite_amounts() {
    assert!(ImperialUnit::Cup.to_metric(-1.0).is_none());
    assert!(ImperialUnit::Ounce.to_metric(f64::NAN).is_none());
    assert!(ImperialUnit::Pound.to_metric(f64::INFINITY).is_none());
}

#[test]
fn rejects_negative_and_non_finite_amounts() {
    assert!(Quantity::new(-1.0, Unit::Gram).is_none());
    assert!(Quantity::new(f64::NAN, Unit::Gram).is_none());
    assert!(Quantity::new(f64::INFINITY, Unit::Litre).is_none());
    assert!(Quantity::new(f64::NEG_INFINITY, Unit::Piece).is_none());
}

#[test]
fn accepts_zero() {
    assert_is(quantity(0.0, Unit::Gram), 0.0, Unit::Gram);
}

#[test]
fn unitless_amounts_are_kept_as_they_are() {
    assert_is(quantity(2.0, Unit::Pinch).display(), 2.0, Unit::Pinch);
    assert_is(
        quantity(1.0, Unit::ToTaste)
            .convert_to(Unit::ToTaste)
            .unwrap(),
        1.0,
        Unit::ToTaste,
    );
}
