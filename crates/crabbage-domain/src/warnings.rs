//! The fixed dietary warnings a recipe can carry, for people who must avoid more than allergens:
//! what is advised against during pregnancy.

/// Something in a recipe that some diets advise against.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Warning {
    /// Raw or lightly cooked egg.
    RawEgg,
    /// Raw fish or shellfish.
    RawFish,
    /// Rare or raw meat.
    RareMeat,
    /// Cheese made from unpasteurised milk.
    UnpasteurisedCheese,
    /// Liver and liver products.
    Liver,
    /// Alcohol that is not cooked off.
    Alcohol,
}

impl Warning {
    /// Every warning, in the order forms and lists show them.
    pub const ALL: [Self; 6] = [
        Self::RawEgg,
        Self::RawFish,
        Self::RareMeat,
        Self::UnpasteurisedCheese,
        Self::Liver,
        Self::Alcohol,
    ];

    /// The stable code stored and sent in forms.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::RawEgg => "raw_egg",
            Self::RawFish => "raw_fish",
            Self::RareMeat => "rare_meat",
            Self::UnpasteurisedCheese => "unpasteurised_cheese",
            Self::Liver => "liver",
            Self::Alcohol => "alcohol",
        }
    }

    /// The warning with `code`, ignoring case and surrounding spaces. `None` for anything else.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        let code = code.trim();
        Self::ALL
            .into_iter()
            .find(|warning| warning.code().eq_ignore_ascii_case(code))
    }

    /// The warnings named by `codes`, in [`Warning::ALL`] order, each once. Unknown codes are left
    /// out.
    #[must_use]
    pub fn from_codes<'a>(codes: impl IntoIterator<Item = &'a str>) -> Vec<Self> {
        let mut warnings: Vec<Self> = codes.into_iter().filter_map(Self::from_code).collect();
        warnings.sort_unstable();
        warnings.dedup();
        warnings
    }
}
