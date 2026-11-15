//! The names of the meal types, from the locale.

use crabbage_domain::meal_types::MealType;

/// The locale key of a meal type's name.
pub(crate) const fn meal_key(meal: MealType) -> &'static str {
    match meal {
        MealType::Breakfast => "meal.breakfast",
        MealType::Lunch => "meal.lunch",
        MealType::Dinner => "meal.dinner",
        MealType::Dessert => "meal.dessert",
        MealType::Side => "meal.side",
        MealType::Snack => "meal.snack",
        MealType::Drink => "meal.drink",
    }
}
