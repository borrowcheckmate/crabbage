//! The fixed allergens an ingredient can contain: the fourteen the EU requires on labels, and meat.
//! Fish, milk and eggs together with meat tell vegetarian and vegan diets apart.

/// Something an ingredient contains that some people avoid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Allergen {
    /// Cereals containing gluten.
    Gluten,
    /// Crustaceans, such as prawns.
    Crustaceans,
    /// Eggs.
    Eggs,
    /// Fish.
    Fish,
    /// Peanuts.
    Peanuts,
    /// Soybeans.
    Soy,
    /// Milk, lactose included.
    Milk,
    /// Tree nuts, such as almonds and walnuts.
    TreeNuts,
    /// Celery and celeriac.
    Celery,
    /// Mustard.
    Mustard,
    /// Sesame seeds.
    Sesame,
    /// Sulphur dioxide and sulphites.
    Sulphites,
    /// Lupin.
    Lupin,
    /// Molluscs, such as mussels and squid.
    Molluscs,
    /// Meat, which vegetarians and vegans avoid.
    Meat,
}

impl Allergen {
    /// Every allergen, in the order forms and lists show them.
    pub const ALL: [Self; 15] = [
        Self::Gluten,
        Self::Crustaceans,
        Self::Eggs,
        Self::Fish,
        Self::Peanuts,
        Self::Soy,
        Self::Milk,
        Self::TreeNuts,
        Self::Celery,
        Self::Mustard,
        Self::Sesame,
        Self::Sulphites,
        Self::Lupin,
        Self::Molluscs,
        Self::Meat,
    ];

    /// The stable code stored and sent in forms.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Gluten => "gluten",
            Self::Crustaceans => "crustaceans",
            Self::Eggs => "eggs",
            Self::Fish => "fish",
            Self::Peanuts => "peanuts",
            Self::Soy => "soy",
            Self::Milk => "milk",
            Self::TreeNuts => "tree_nuts",
            Self::Celery => "celery",
            Self::Mustard => "mustard",
            Self::Sesame => "sesame",
            Self::Sulphites => "sulphites",
            Self::Lupin => "lupin",
            Self::Molluscs => "molluscs",
            Self::Meat => "meat",
        }
    }

    /// The allergen with `code`, ignoring case and surrounding spaces. `None` for anything else.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        let code = code.trim();
        Self::ALL
            .into_iter()
            .find(|allergen| allergen.code().eq_ignore_ascii_case(code))
    }

    /// The allergens named by `codes`, in [`Allergen::ALL`] order, each once. Unknown codes are
    /// left out.
    #[must_use]
    pub fn from_codes<'a>(codes: impl IntoIterator<Item = &'a str>) -> Vec<Self> {
        let mut allergens: Vec<Self> = codes.into_iter().filter_map(Self::from_code).collect();
        allergens.sort_unstable();
        allergens.dedup();
        allergens
    }
}
