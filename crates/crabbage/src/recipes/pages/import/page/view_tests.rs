use super::{encode_query, format_amount};

#[test]
fn retry_urls_encode_query_delimiters_and_unicode_bytes() {
    assert_eq!(
        encode_query("https://example.test/?a=one&b=two"),
        "https%3A%2F%2Fexample.test%2F%3Fa%3Done%26b%3Dtwo"
    );
    assert_eq!(encode_query("café"), "caf%C3%A9");
}

#[test]
fn ingredient_amounts_keep_units_and_unspecified_values() {
    assert_eq!(format_amount("2", "piece"), "2");
    assert_eq!(format_amount("2", "tbsp"), "2 tbsp");
    assert_eq!(format_amount("", "to taste"), "to taste");
    assert_eq!(format_amount("", ""), "Unspecified");
}
