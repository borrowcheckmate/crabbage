//! Amounts with a unit: conversion, addition and the readable display unit.

use super::{Dimension, Unit};

const KILO: f64 = 1000.0;

/// A finite, non-negative amount in a unit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quantity {
    amount: f64,
    unit: Unit,
}

impl Quantity {
    /// Creates a quantity. `None` when `amount` is negative or not finite.
    #[must_use]
    pub fn new(amount: f64, unit: Unit) -> Option<Self> {
        (amount.is_finite() && amount >= 0.0).then_some(Self { amount, unit })
    }

    /// The amount, in the unit returned by [`Quantity::unit`].
    #[must_use]
    pub const fn amount(self) -> f64 {
        self.amount
    }

    /// The unit of the amount.
    #[must_use]
    pub const fn unit(self) -> Unit {
        self.unit
    }

    /// Converts to `unit` through the base unit of the dimension.
    ///
    /// `None` when `unit` measures another dimension. Unitless units (pinch, to taste) convert
    /// only to themselves. `None` also when the result is not finite.
    #[must_use]
    pub fn convert_to(self, unit: Unit) -> Option<Self> {
        if self.unit == unit {
            return Some(self);
        }
        if self.unit.dimension() != unit.dimension() {
            return None;
        }
        let base = self.amount * self.unit.factor()?;
        Self::new(base / unit.factor()?, unit)
    }

    /// Adds two quantities and returns the sum in its display unit.
    ///
    /// Units that are the same are added as they are, so two teaspoons stay teaspoons and two
    /// pinches make two pinches. Units of one dimension that differ are added in the base unit
    /// (g, ml or piece). `None` when the dimensions differ, when the units are unitless and not
    /// equal, or when the sum is not finite.
    #[must_use]
    pub fn checked_add(self, other: Self) -> Option<Self> {
        if self.unit == other.unit {
            return Self::new(self.amount + other.amount, self.unit).map(Self::display);
        }
        let base = base_unit(self.unit.dimension())?;
        let sum = self.convert_to(base)?.amount + other.convert_to(base)?.amount;
        Self::new(sum, base).map(Self::display)
    }

    /// The readable form of the amount.
    ///
    /// - Mass: grams below 1000 g, kilograms from 1000 g up.
    /// - Volume: millilitres below 1000 ml, litres from 1000 ml up. Teaspoon and tablespoon
    ///   amounts are kept as they are, because spoons read better than millilitres.
    /// - Count and unitless amounts are returned unchanged.
    #[must_use]
    pub fn display(self) -> Self {
        let (small, large) = match self.unit.dimension() {
            Dimension::Mass => (Unit::Gram, Unit::Kilogram),
            Dimension::Volume if matches!(self.unit, Unit::Teaspoon | Unit::Tablespoon) => {
                return self;
            }
            Dimension::Volume => (Unit::Millilitre, Unit::Litre),
            Dimension::Count | Dimension::Unitless => return self,
        };
        let Some(in_small) = self.convert_to(small) else {
            return self;
        };
        if in_small.amount < KILO {
            in_small
        } else {
            Self {
                amount: in_small.amount / KILO,
                unit: large,
            }
        }
    }
}

const fn base_unit(dimension: Dimension) -> Option<Unit> {
    match dimension {
        Dimension::Mass => Some(Unit::Gram),
        Dimension::Volume => Some(Unit::Millilitre),
        Dimension::Count => Some(Unit::Piece),
        Dimension::Unitless => None,
    }
}
