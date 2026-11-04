//! Scaling amounts by servings: ratio, readable rounding, unitless amounts, identity and bounds.
#![expect(
    clippy::unwrap_used,
    reason = "helpers build quantities and servings from literals known to be valid"
)]

use crabbage_domain::quantity::{Quantity, Unit};
use crabbage_domain::scaling::{Servings, scale};

const EPSILON: f64 = 1e-9;

fn quantity(amount: f64, unit: Unit) -> Quantity {
    Quantity::new(amount, unit).unwrap()
}

const fn servings(count: u32) -> Servings {
    Servings::new(count).unwrap()
}

fn scaled(amount: f64, unit: Unit, from: u32, to: u32) -> Quantity {
    scale(quantity(amount, unit), servings(from), servings(to))
}

fn assert_is(actual: Quantity, amount: f64, unit: Unit) {
    assert!(
        (actual.amount() - amount).abs() < EPSILON,
        "{} != {amount}",
        actual.amount()
    );
    assert_eq!(actual.unit(), unit);
}

#[test]
fn zero_servings_are_rejected() {
    assert_eq!(Servings::new(0), None);
    assert_eq!(Servings::new(1).map(Servings::get), Some(1));
}

#[test]
fn amounts_scale_by_the_servings_ratio() {
    assert_is(scaled(200.0, Unit::Gram, 4, 2), 100.0, Unit::Gram);
    assert_is(scaled(1.0, Unit::Teaspoon, 2, 4), 2.0, Unit::Teaspoon);
    assert_is(scaled(3.0, Unit::Piece, 2, 3), 4.5, Unit::Piece);
}

#[test]
fn grams_round_to_five_below_one_thousand() {
    assert_is(scaled(100.0, Unit::Gram, 3, 10), 335.0, Unit::Gram);
}

#[test]
fn kilograms_round_to_five_hundredths() {
    assert_is(scaled(1185.0, Unit::Gram, 1, 2), 2.35, Unit::Kilogram);
}

#[test]
fn teaspoons_round_to_quarters() {
    assert_is(scaled(0.165, Unit::Teaspoon, 1, 2), 0.25, Unit::Teaspoon);
}

#[test]
fn millilitres_round_to_half_below_ten() {
    assert_is(scaled(3.65, Unit::Millilitre, 1, 2), 7.5, Unit::Millilitre);
}

#[test]
fn grams_round_to_one_below_one_hundred() {
    assert_is(scaled(30.4, Unit::Gram, 1, 2), 61.0, Unit::Gram);
    assert_is(scaled(15.0, Unit::Gram, 2, 3), 23.0, Unit::Gram);
}

#[test]
fn a_tiny_nonzero_amount_rounds_to_the_smallest_step_not_zero() {
    assert_is(scaled(0.1, Unit::Gram, 2, 1), 0.5, Unit::Gram);
}

#[test]
fn unitless_amounts_are_not_scaled() {
    assert_eq!(scaled(1.0, Unit::Pinch, 2, 6), quantity(1.0, Unit::Pinch));
    assert_eq!(
        scaled(0.0, Unit::ToTaste, 2, 6),
        quantity(0.0, Unit::ToTaste)
    );
}

#[test]
fn scaling_to_the_original_servings_is_the_identity() {
    let odd = quantity(333.333, Unit::Gram);
    assert_eq!(scale(odd, servings(3), servings(3)), odd);
    let spoons = quantity(0.33, Unit::Teaspoon);
    assert_eq!(scale(spoons, servings(2), servings(2)), spoons);
}

#[test]
fn rounding_up_to_a_display_boundary_shows_the_next_unit() {
    assert_is(scaled(499.0, Unit::Gram, 1, 2), 1.0, Unit::Kilogram);
    assert_is(scaled(499.0, Unit::Millilitre, 1, 2), 1.0, Unit::Litre);
}

#[test]
fn a_scaled_value_beyond_the_float_range_is_returned_unchanged() {
    let huge = quantity(f64::MAX, Unit::Gram);
    assert_eq!(scale(huge, servings(1), servings(2)), huge);
}
