use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use super::{Language, Messages};

fn locales_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("locales")
}

fn keys_of(path: &Path) -> BTreeSet<String> {
    let text = fs::read_to_string(path).unwrap();
    let messages: HashMap<String, String> = serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("{} is not a flat JSON object: {error}", path.display()));
    messages.into_keys().collect()
}

fn locale_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(locales_dir())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    files
}

#[test]
fn every_locale_file_has_the_same_keys_as_english() {
    let english = keys_of(&locales_dir().join("en.json"));
    assert!(!english.is_empty());
    for path in locale_files() {
        let code = path.file_stem().unwrap().to_str().unwrap();
        assert!(
            Language::from_code(code).is_some(),
            "{code} has no Language variant"
        );
        assert_eq!(keys_of(&path), english, "{}", path.display());
    }
}

#[test]
fn every_key_is_translated_in_every_language() {
    let keys = keys_of(&locales_dir().join("en.json"));
    for language in [Language::En, Language::Nl] {
        let messages = Messages::for_language(language);
        for key in &keys {
            assert_ne!(
                messages.get(key),
                key.as_str(),
                "{key} is missing in {language:?}"
            );
        }
    }
}

#[test]
fn dutch_text_differs_from_english_where_it_is_translated() {
    assert_eq!(
        Messages::for_language(Language::Nl).get("app.sign_out"),
        "Uitloggen"
    );
    assert_eq!(
        Messages::for_language(Language::En).get("app.sign_out"),
        "Sign out"
    );
}

#[test]
fn missing_keys_are_returned_as_the_key() {
    let messages = Messages::from_json(r#"{"known": "text"}"#);
    assert_eq!(messages.get("unknown.key"), "unknown.key");
    assert_eq!(messages.get("known"), "text");
}

#[test]
fn language_codes_round_trip() {
    for language in [Language::En, Language::Nl] {
        assert_eq!(Language::from_code(language.code()), Some(language));
    }
    assert_eq!(Language::from_code("de"), None);
    assert_eq!(Language::from_code("EN"), None);
}

#[test]
fn format_fills_named_placeholders() {
    let messages = Messages::from_json(r#"{"cart": "{count} items for {name}"}"#);
    assert_eq!(
        messages.format("cart", &[("count", "3"), ("name", "Ada")]),
        "3 items for Ada"
    );
}

#[test]
fn format_keeps_unknown_placeholders_and_unclosed_braces() {
    let messages = Messages::from_json(r#"{"cart": "{count} for {unknown} and {open"}"#);
    assert_eq!(
        messages.format("cart", &[("count", "2")]),
        "2 for {unknown} and {open"
    );
}

#[test]
fn format_does_not_fill_placeholders_inside_values() {
    let messages = Messages::from_json(r#"{"cart": "{a} {b}"}"#);
    assert_eq!(
        messages.format("cart", &[("a", "{b}"), ("b", "x")]),
        "{b} x"
    );
}

#[test]
fn format_returns_text_without_placeholders_unchanged() {
    let messages = Messages::from_json(r#"{"title": "crabbage"}"#);
    assert_eq!(messages.format("title", &[("count", "1")]), "crabbage");
    assert_eq!(messages.format("missing", &[]), "missing");
}
