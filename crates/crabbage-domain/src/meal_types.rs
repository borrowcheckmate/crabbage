//! The fixed meal types a recipe can be marked with, so the app can rely on them.

/// When a recipe is eaten. A recipe can have several.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MealType {
    /// Breakfast.
    Breakfast,
    /// Lunch.
    Lunch,
    /// Dinner.
    Dinner,
    /// Dessert.
    Dessert,
    /// A side dish.
    Side,
    /// A snack.
    Snack,
    /// A drink.
    Drink,
}

impl MealType {
    /// Every meal type, in the order forms and filters show them.
    pub const ALL: [Self; 7] = [
        Self::Breakfast,
        Self::Lunch,
        Self::Dinner,
        Self::Dessert,
        Self::Side,
        Self::Snack,
        Self::Drink,
    ];

    /// The stable code stored and sent in forms: `breakfast`, `lunch`, `dinner`, `dessert`,
    /// `side`, `snack` or `drink`.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Breakfast => "breakfast",
            Self::Lunch => "lunch",
            Self::Dinner => "dinner",
            Self::Dessert => "dessert",
            Self::Side => "side",
            Self::Snack => "snack",
            Self::Drink => "drink",
        }
    }

    /// The meal type with `code`, ignoring case and surrounding spaces. `None` for anything else.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        let code = code.trim();
        Self::ALL
            .into_iter()
            .find(|meal| meal.code().eq_ignore_ascii_case(code))
    }

    /// The meal types named by `codes`, in [`MealType::ALL`] order, each once. Unknown codes are
    /// left out.
    #[must_use]
    pub fn from_codes<'a>(codes: impl IntoIterator<Item = &'a str>) -> Vec<Self> {
        let mut meals: Vec<Self> = codes.into_iter().filter_map(Self::from_code).collect();
        meals.sort_unstable();
        meals.dedup();
        meals
    }
}
