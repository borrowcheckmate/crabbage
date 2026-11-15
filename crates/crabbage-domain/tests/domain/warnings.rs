use crabbage_domain::warnings::Warning;

#[test]
fn every_warning_round_trips_through_its_code() {
    for warning in Warning::ALL {
        assert_eq!(Warning::from_code(warning.code()), Some(warning));
    }
    assert_eq!(Warning::from_code(" Raw_Egg "), Some(Warning::RawEgg));
    assert_eq!(Warning::from_code("egg"), None);
}

#[test]
fn codes_become_known_warnings_in_order_and_once() {
    assert_eq!(
        Warning::from_codes(["alcohol", "liver", "bogus", "LIVER", "raw_egg"]),
        [Warning::RawEgg, Warning::Liver, Warning::Alcohol]
    );
}
