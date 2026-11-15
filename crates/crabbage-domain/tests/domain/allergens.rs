use crabbage_domain::allergens::Allergen;

#[test]
fn every_allergen_round_trips_through_its_code() {
    for allergen in Allergen::ALL {
        assert_eq!(Allergen::from_code(allergen.code()), Some(allergen));
    }
    assert_eq!(Allergen::from_code(" Tree_Nuts "), Some(Allergen::TreeNuts));
    assert_eq!(Allergen::from_code("nuts"), None);
}

#[test]
fn codes_become_known_allergens_in_order_and_once() {
    assert_eq!(
        Allergen::from_codes(["meat", "milk", "bogus", "MILK", "gluten"]),
        [Allergen::Gluten, Allergen::Milk, Allergen::Meat]
    );
}
