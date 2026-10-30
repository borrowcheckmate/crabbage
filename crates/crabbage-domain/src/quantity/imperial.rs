//! Imperial units used in recipes, converted to metric on input.

use super::{Quantity, Unit};

/// An imperial unit that recipes may be written in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImperialUnit {
    /// Avoirdupois ounce: 28.349523125 g.
    Ounce,
    /// Pound: 453.59237 g.
    Pound,
    /// US cup: 240 ml.
    Cup,
    /// US fluid ounce: 29.5735295625 ml.
    FluidOunce,
}

impl ImperialUnit {
    /// Converts `amount` of this unit to metric and returns it in its display unit, so
    /// 16 oz is 453.59 g and 3 lb is 1.36 kg.
    ///
    /// `None` when `amount` is negative or not finite.
    #[must_use]
    pub fn to_metric(self, amount: f64) -> Option<Quantity> {
        let (factor, unit) = match self {
            Self::Ounce => (28.349_523_125, Unit::Gram),
            Self::Pound => (453.59237, Unit::Gram),
            Self::Cup => (240.0, Unit::Millilitre),
            Self::FluidOunce => (29.573_529_562_5, Unit::Millilitre),
        };
        Quantity::new(amount * factor, unit).map(Quantity::display)
    }
}
