//! Guards the base stylesheet `style/main.css` with plain string checks: the dark colour scheme,
//! touch-sized controls and the body width limit. These are the properties the mobile layout relies
//! on, so a rewrite that drops one fails here.

#![expect(
    clippy::unwrap_used,
    reason = "the test stops when the stylesheet cannot be read"
)]

use std::fs;

/// The stylesheet as the build serves it, read from its source file.
fn stylesheet() -> String {
    fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/style/main.css")).unwrap()
}

/// The `min-height` in pixels of the first rule that styles every selector in `selectors`.
fn min_height_of(css: &str, selectors: &[&str]) -> Option<u32> {
    css.split('}').find_map(|block| {
        let (selector, body) = block.split_once('{')?;
        let listed: Vec<&str> = selector.split(',').map(str::trim).collect();
        if !selectors.iter().all(|wanted| listed.contains(wanted)) {
            return None;
        }
        let value = body.split("min-height:").nth(1)?;
        value.trim_start().split("px").next()?.trim().parse().ok()
    })
}

#[test]
fn stylesheet_follows_the_system_colour_scheme() {
    assert!(
        stylesheet().contains("prefers-color-scheme: dark"),
        "dark colours must follow the system setting"
    );
}

#[test]
fn buttons_and_inputs_are_at_least_44_pixels_tall() {
    let css = stylesheet();
    for selectors in [&["input", "button"][..], &["input", "select", "button"][..]] {
        let height = min_height_of(&css, selectors);
        assert!(
            height.is_some_and(|px| px >= 44),
            "{selectors:?} must have min-height of at least 44px, found {height:?}"
        );
    }
}

#[test]
fn body_has_a_maximum_width() {
    let css = stylesheet();
    let body = css.split('}').find_map(|block| {
        block
            .split_once('{')
            .filter(|(selector, _)| selector.trim() == "body")
    });
    assert!(
        body.is_some_and(|(_, declarations)| declarations.contains("max-width:")),
        "body must limit its width so text stays readable on large screens"
    );
}
