//! Scaling of recipe amounts from one number of servings to another.

use std::num::NonZeroU32;

use crate::quantity::{Quantity, Unit};

/// A number of servings. Always at least one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Servings(NonZeroU32);

impl Servings {
    /// Creates servings. `None` for zero.
    #[must_use]
    pub const fn new(count: u32) -> Option<Self> {
        match NonZeroU32::new(count) {
            Some(count) => Some(Self(count)),
            None => None,
        }
    }

    /// The number of servings.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

/// Scales `quantity` from `from` servings to `to` servings.
///
/// - Equal servings return the quantity unchanged: no rounding and no display change.
/// - Unitless amounts (pinch, to taste) are returned unchanged.
/// - Otherwise the amount is multiplied by `to / from`, shown in its display unit (see
///   [`Quantity::display`]) and rounded to a step chosen by that unit:
///   - grams and millilitres: 0.5 below 10, 1 below 100, 5 otherwise;
///   - kilograms and litres: 0.05;
///   - teaspoons and tablespoons: 0.25;
///   - pieces: 0.5.
/// - A non-zero amount never rounds to zero: the step is used instead.
/// - Rounding can reach a display boundary (998 g becomes 1000 g), so the rounded amount is
///   displayed again (1 kg).
/// - An amount whose scaled value is not finite (beyond the range of `f64`) is returned
///   unchanged.
#[must_use]
pub fn scale(quantity: Quantity, from: Servings, to: Servings) -> Quantity {
    if from == to {
        return quantity;
    }
    let ratio = f64::from(to.get()) / f64::from(from.get());
    let Some(scaled) = Quantity::new(quantity.amount() * ratio, quantity.unit()) else {
        return quantity;
    };
    let shown = scaled.display();
    let Some(step) = step(shown.unit(), shown.amount()) else {
        return quantity;
    };
    Quantity::new(round_to(shown.amount(), step), shown.unit()).map_or(quantity, Quantity::display)
}

/// The rounding step for a displayed amount. `None` for unitless units.
const fn step(unit: Unit, amount: f64) -> Option<f64> {
    match unit {
        Unit::Gram | Unit::Millilitre => Some(if amount < 10.0 {
            0.5
        } else if amount < 100.0 {
            1.0
        } else {
            5.0
        }),
        Unit::Kilogram | Unit::Litre => Some(0.05),
        Unit::Teaspoon | Unit::Tablespoon => Some(0.25),
        Unit::Piece => Some(0.5),
        Unit::Pinch | Unit::ToTaste => None,
    }
}

/// Rounds to the nearest multiple of `step`, but never to zero when `amount` is positive.
fn round_to(amount: f64, step: f64) -> f64 {
    let rounded = (amount / step).round() * step;
    if amount > 0.0 && rounded <= 0.0 {
        step
    } else {
        rounded
    }
}
