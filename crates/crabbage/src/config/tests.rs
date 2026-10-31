use std::collections::HashMap;
use std::path::PathBuf;

use super::Config;
use crate::i18n::Language;

fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let map: HashMap<String, String> = pairs
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect();
    move |name| map.get(name).cloned()
}

const REQUIRED: [(&str, &str); 4] = [
    ("CRABBAGE_PUBLIC_URL", "https://recipes.example.com"),
    ("OIDC_ISSUER_URL", "https://id.example.com"),
    ("OIDC_CLIENT_ID", "crabbage"),
    ("OIDC_CLIENT_SECRET", "secret"),
];

fn with_required(extra: &[(&'static str, &'static str)]) -> Vec<(&'static str, &'static str)> {
    REQUIRED
        .iter()
        .copied()
        .chain(extra.iter().copied())
        .collect()
}

fn error_text(pairs: &[(&str, &str)]) -> String {
    Config::from_lookup(lookup(pairs)).unwrap_err().to_string()
}

#[test]
fn missing_required_vars_are_all_named() {
    let text = error_text(&[]);
    for name in [
        "CRABBAGE_PUBLIC_URL",
        "OIDC_ISSUER_URL",
        "OIDC_CLIENT_ID",
        "OIDC_CLIENT_SECRET",
    ] {
        assert!(text.contains(name), "{name} missing from {text}");
    }
}

#[test]
fn empty_values_count_as_unset() {
    let text = error_text(&[("OIDC_CLIENT_ID", "")]);
    assert!(text.contains("OIDC_CLIENT_ID is not set"), "{text}");
}

#[test]
fn defaults_apply_when_optional_vars_are_unset() {
    let config = Config::from_lookup(lookup(&REQUIRED)).unwrap();
    assert_eq!(config.language, Language::En);
    assert_eq!(config.data_dir, PathBuf::from("/data"));
    assert_eq!(config.admin_group, "admin");
    assert_eq!(config.household_group_prefix, "crabbage-household-");
    assert_eq!(config.public_url.as_str(), "https://recipes.example.com/");
    assert_eq!(config.oidc_issuer_url, "https://id.example.com");
    assert_eq!(config.oidc_client_id, "crabbage");
    assert_eq!(config.oidc_client_secret, "secret");
    assert!(!config.import_allow_private);
}

#[test]
fn import_allow_private_reads_true_and_false() {
    for (value, expected) in [("true", true), ("false", false)] {
        let pairs = with_required(&[("CRABBAGE_IMPORT_ALLOW_PRIVATE", value)]);
        let config = Config::from_lookup(lookup(&pairs)).unwrap();
        assert_eq!(config.import_allow_private, expected);
    }
}

#[test]
fn import_allow_private_rejects_other_values() {
    let text = error_text(&with_required(&[("CRABBAGE_IMPORT_ALLOW_PRIVATE", "yes")]));
    assert!(
        text.contains("CRABBAGE_IMPORT_ALLOW_PRIVATE is invalid: expected true or false, not yes"),
        "{text}"
    );
}

#[test]
fn optional_vars_override_defaults() {
    let pairs = with_required(&[
        ("CRABBAGE_LANGUAGE", "nl"),
        ("CRABBAGE_DATA_DIR", "/srv/crabbage"),
        ("CRABBAGE_ADMIN_GROUP", "chefs"),
        ("CRABBAGE_HOUSEHOLD_GROUP_PREFIX", "home-"),
    ]);
    let config = Config::from_lookup(lookup(&pairs)).unwrap();
    assert_eq!(config.language, Language::Nl);
    assert_eq!(config.data_dir, PathBuf::from("/srv/crabbage"));
    assert_eq!(config.admin_group, "chefs");
    assert_eq!(config.household_group_prefix, "home-");
}

#[test]
fn invalid_language_is_rejected() {
    let text = error_text(&with_required(&[("CRABBAGE_LANGUAGE", "fr")]));
    assert!(text.contains("CRABBAGE_LANGUAGE is invalid"), "{text}");
    assert!(text.contains("not fr"), "{text}");
}

#[test]
fn unparseable_url_is_rejected() {
    let text = error_text(&with_required(&[("OIDC_ISSUER_URL", "not a url")]));
    assert!(text.contains("OIDC_ISSUER_URL is invalid"), "{text}");
}

#[test]
fn non_http_url_is_rejected() {
    let text = error_text(&with_required(&[(
        "CRABBAGE_PUBLIC_URL",
        "ftp://example.com",
    )]));
    assert!(text.contains("CRABBAGE_PUBLIC_URL is invalid"), "{text}");
    assert!(
        text.contains("scheme must be http or https, not ftp"),
        "{text}"
    );
}

#[test]
fn every_problem_is_reported_together() {
    let text = error_text(&[("CRABBAGE_LANGUAGE", "fr")]);
    for needle in [
        "CRABBAGE_LANGUAGE is invalid",
        "CRABBAGE_PUBLIC_URL is not set",
        "OIDC_CLIENT_SECRET is not set",
    ] {
        assert!(text.contains(needle), "{needle} missing from {text}");
    }
}

#[test]
fn debug_redacts_secrets() {
    let config = Config::from_lookup(lookup(&REQUIRED)).unwrap();
    let debug = format!("{config:?}");
    assert!(
        debug.contains("oidc_client_secret: \"<redacted>\""),
        "{debug}"
    );
    assert!(!debug.contains("secret\""), "{debug}");
}
