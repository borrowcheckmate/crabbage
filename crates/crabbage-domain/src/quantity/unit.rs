//! Units of measure and the dimension each one measures.

/// What a unit measures. Amounts convert only within a dimension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dimension {
    /// Weight: grams and kilograms.
    Mass,
    /// Capacity: millilitres, litres and spoons.
    Volume,
    /// Whole items: pieces.
    Count,
    /// Amounts without a measure, such as a pinch or to taste.
    Unitless,
}

/// A unit of measure used by recipe amounts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    /// Gram, the base unit of mass.
    Gram,
    /// Kilogram: 1000 g.
    Kilogram,
    /// Millilitre, the base unit of volume.
    Millilitre,
    /// Litre: 1000 ml.
    Litre,
    /// Teaspoon: 5 ml.
    Teaspoon,
    /// Tablespoon: 15 ml.
    Tablespoon,
    /// A single item. The base unit of count.
    Piece,
    /// A pinch. Unitless.
    Pinch,
    /// Amount left to the cook. Unitless.
    ToTaste,
}

impl Unit {
    /// Every unit, in the order a form lists them.
    pub const ALL: [Self; 9] = [
        Self::Gram,
        Self::Kilogram,
        Self::Millilitre,
        Self::Litre,
        Self::Teaspoon,
        Self::Tablespoon,
        Self::Piece,
        Self::Pinch,
        Self::ToTaste,
    ];

    /// The unit whose [`symbol`](Self::symbol) is `symbol`, or `None` when there is none.
    #[must_use]
    pub fn from_symbol(symbol: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|unit| unit.symbol() == symbol)
    }

    /// The dimension this unit measures.
    #[must_use]
    pub const fn dimension(self) -> Dimension {
        match self {
            Self::Gram | Self::Kilogram => Dimension::Mass,
            Self::Millilitre | Self::Litre | Self::Teaspoon | Self::Tablespoon => Dimension::Volume,
            Self::Piece => Dimension::Count,
            Self::Pinch | Self::ToTaste => Dimension::Unitless,
        }
    }

    /// How many base units make one of this unit: grams for mass, millilitres for volume and
    /// pieces for count. `None` for unitless units, which have no base unit.
    #[must_use]
    pub const fn factor(self) -> Option<f64> {
        match self {
            Self::Gram | Self::Millilitre | Self::Piece => Some(1.0),
            Self::Kilogram | Self::Litre => Some(1000.0),
            Self::Teaspoon => Some(5.0),
            Self::Tablespoon => Some(15.0),
            Self::Pinch | Self::ToTaste => None,
        }
    }

    /// The short written form of the unit.
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Gram => "g",
            Self::Kilogram => "kg",
            Self::Millilitre => "ml",
            Self::Litre => "l",
            Self::Teaspoon => "tsp",
            Self::Tablespoon => "tbsp",
            Self::Piece => "piece",
            Self::Pinch => "pinch",
            Self::ToTaste => "to taste",
        }
    }
}
