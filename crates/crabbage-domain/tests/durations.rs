//! Durations named in step text, English and Dutch, with the negatives that must not count.

use crabbage_domain::durations::detect_durations;

fn seconds(text: &str) -> Vec<u64> {
    detect_durations(text)
        .iter()
        .map(std::time::Duration::as_secs)
        .collect()
}

#[test]
fn english_minutes_in_their_full_and_short_forms() {
    assert_eq!(seconds("Bake for 25 minutes"), [1500]);
    assert_eq!(seconds("Bake for 25 min"), [1500]);
    assert_eq!(seconds("Stir for 5 mins"), [300]);
    assert_eq!(seconds("Rest for 1 minute"), [60]);
    assert_eq!(seconds("Simmer for 10-minute intervals"), [600]);
}

#[test]
fn english_hours() {
    assert_eq!(seconds("Roast for 1 hour"), [3600]);
    assert_eq!(seconds("Proof for 2 hours"), [7200]);
    assert_eq!(seconds("Proof for 1.5 hours"), [5400]);
    assert_eq!(seconds("Chill for 2 hrs"), [7200]);
}

#[test]
fn english_seconds() {
    assert_eq!(seconds("Beat for 90 seconds"), [90]);
    assert_eq!(seconds("Flip after 30 sec"), [30]);
    assert_eq!(seconds("Flip after 45 secs"), [45]);
}

#[test]
fn english_decimal_comma_and_fractions_of_a_unit() {
    assert_eq!(seconds("Proof for 1,5 hours"), [5400]);
    assert_eq!(seconds("Simmer for 1.5 min"), [90]);
    assert_eq!(seconds("Let it stand for a minute"), [60]);
    assert_eq!(seconds("Let it rest for an hour"), [3600]);
    assert_eq!(seconds("Let it rest for a half hour"), [1800]);
}

#[test]
fn english_ranges_give_the_upper_bound() {
    assert_eq!(seconds("Fry 10-15 minutes"), [900]);
    assert_eq!(seconds("Fry 10 to 15 minutes"), [900]);
    assert_eq!(seconds("Fry 10–15 minutes"), [900]);
    assert_eq!(seconds("Bake 1 to 2 hours"), [7200]);
}

#[test]
fn english_combined_units_make_one_duration() {
    assert_eq!(seconds("Bake for 1 hour 30 minutes"), [5400]);
    assert_eq!(seconds("Bake for 1 hour and 30 minutes"), [5400]);
    assert_eq!(seconds("Bake for 1 hour 30 minutes 20 seconds"), [5420]);
    assert_eq!(seconds("Rise for 1 hour 10-15 minutes"), [3600, 900]);
}

#[test]
fn dutch_minutes_hours_and_seconds() {
    assert_eq!(seconds("Laat 25 minuten rusten"), [1500]);
    assert_eq!(seconds("Laat 1 minuut staan"), [60]);
    assert_eq!(seconds("Laat 1 uur rusten"), [3600]);
    assert_eq!(seconds("Laat 2 uur rijzen"), [7200]);
    assert_eq!(seconds("Laat 2 uren rijzen"), [7200]);
    assert_eq!(seconds("Klop 30 seconden"), [30]);
    assert_eq!(seconds("Klop 1 seconde"), [1]);
}

#[test]
fn dutch_fractions_and_half_units() {
    assert_eq!(seconds("Laat 1,5 uur rusten"), [5400]);
    assert_eq!(seconds("Laat anderhalf uur rusten"), [5400]);
    assert_eq!(seconds("Laat anderhalve minuut staan"), [90]);
    assert_eq!(seconds("Laat een half uur rusten"), [1800]);
    assert_eq!(seconds("Laat een uur rusten"), [3600]);
}

#[test]
fn dutch_ranges_give_the_upper_bound() {
    assert_eq!(seconds("Bak 10-15 minuten"), [900]);
    assert_eq!(seconds("Bak 10 à 15 minuten"), [900]);
    assert_eq!(seconds("Bak 10 tot 15 minuten"), [900]);
}

#[test]
fn dutch_combined_units_make_one_duration() {
    assert_eq!(seconds("Laat 1 uur en 30 minuten rijzen"), [5400]);
    assert_eq!(seconds("Laat 1 uur, 30 minuten rijzen"), [5400]);
    assert_eq!(seconds("Laat 1 uur 30 minuten rijzen"), [5400]);
}

#[test]
fn matching_ignores_case() {
    assert_eq!(seconds("BAKE FOR 25 MINUTES"), [1500]);
    assert_eq!(seconds("Preheat for 2 Hours"), [7200]);
    assert_eq!(seconds("Laat 1 UUR RUSTEN"), [3600]);
}

#[test]
fn units_must_be_whole_words() {
    assert_eq!(seconds("Use the minimum amount"), []);
    assert_eq!(seconds("Bake 25 minimum"), []);
    assert_eq!(seconds("Serve by the hourglass"), []);
    assert_eq!(seconds("Add 2 hourly tips"), []);
    assert_eq!(seconds("Laat 1 uurtje staan"), []);
    assert_eq!(seconds("Sec"), []);
}

#[test]
fn a_number_without_a_unit_is_not_a_duration() {
    assert_eq!(seconds("Add 2 eggs and 250 g flour"), []);
    assert_eq!(seconds("Heat the oven to 180 degrees"), []);
    assert_eq!(seconds("Stir for a few minutes"), []);
    assert_eq!(seconds("Serve 25 cm tins"), []);
    assert_eq!(seconds("minutes later"), []);
    assert_eq!(seconds(""), []);
}

#[test]
fn zero_and_overflowing_amounts_are_not_durations() {
    assert_eq!(seconds("Rest 0 minutes"), []);
    assert_eq!(seconds("Rest 0.4 seconds"), []);
    assert_eq!(seconds("Rest 99999999999999999999999 minutes"), []);
}

#[test]
fn units_of_the_same_size_do_not_combine() {
    assert_eq!(seconds("Fry 5 minutes and 10 minutes"), [300, 600]);
    assert_eq!(seconds("Simmer 10 minutes, then bake 1 hour"), [600, 3600]);
    assert_eq!(seconds("Bake 1 hour 30 minutes"), [5400]);
    assert_eq!(seconds("Bake 30 minutes 1 hour"), [1800, 3600]);
}

#[test]
fn amounts_are_taken_from_the_number_next_to_the_unit() {
    assert_eq!(seconds("Add 2 eggs, then bake 25 minutes"), [1500]);
    assert_eq!(seconds("Mix 2. 10 minutes later"), [600]);
    assert_eq!(seconds("Bake 2 cups 5 min"), [300]);
}

#[test]
fn a_recipe_text_with_several_durations_keeps_them_in_order() {
    assert_eq!(
        seconds("Whisk for 2 minutes, rest 1 hour, then fry for 4 minutes"),
        [120, 3600, 240]
    );
}
