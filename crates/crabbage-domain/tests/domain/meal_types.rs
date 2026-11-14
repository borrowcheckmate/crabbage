use crabbage_domain::meal_types::MealType;

#[test]
fn every_meal_type_round_trips_through_its_code() {
    for meal in MealType::ALL {
        assert_eq!(MealType::from_code(meal.code()), Some(meal));
    }
    assert_eq!(MealType::from_code(" Dinner "), Some(MealType::Dinner));
    assert_eq!(MealType::from_code("supper"), None);
}

#[test]
fn codes_become_known_meal_types_in_order_and_once() {
    assert_eq!(
        MealType::from_codes(["drink", "dinner", "bogus", "DINNER", "breakfast"]),
        [MealType::Breakfast, MealType::Dinner, MealType::Drink]
    );
}
