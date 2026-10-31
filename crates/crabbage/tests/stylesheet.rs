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
fn navigation_and_layout_follow_the_visual_contract() {
    let css = stylesheet();
    for contract in [
        "main-workspace (min-width:560px)",
        "min-width:900px",
        "env(safe-area-inset-bottom)",
        "prefers-reduced-motion: reduce",
        ".mobile-nav",
        ".primary-nav",
    ] {
        assert!(
            css.contains(contract),
            "missing responsive shell rule {contract}"
        );
    }
}

#[test]
fn today_and_week_plan_reflow_from_phone_to_paired_panes() {
    let css = stylesheet();
    for contract in [
        ".today-layout { display: grid",
        ".plan-layout { display: grid",
        "@container main-workspace (min-width: 560px)",
        "grid-template-columns: minmax(0, 1.2fr) minmax(17rem, 1fr)",
        "@container main-workspace (min-width: 900px)",
    ] {
        assert!(
            css.contains(contract),
            "missing Today or Plan layout rule {contract}"
        );
    }
}

#[test]
fn navigation_labels_fit_without_breaking_words() {
    let css = stylesheet();
    let desktop = declarations_of(&css, ".primary-nav a").unwrap_or_default();
    let mobile = declarations_of(&css, ".mobile-nav a").unwrap_or_default();
    assert!(css.contains("grid-template-columns:190px minmax(0,1fr)"));
    for declarations in [desktop, mobile] {
        assert!(declarations.contains("overflow-wrap:normal"));
        assert!(declarations.contains("word-break:normal"));
        assert!(declarations.contains("hyphens:manual"));
    }
    assert!(css.contains("text-overflow:ellipsis"));
}

#[test]
fn today_side_heading_aligns_with_the_left_column() {
    let css = stylesheet();
    let heading = declarations_of(&css, ".today-side .today-section-heading").unwrap_or_default();
    assert!(heading.contains("margin-top: 0"));
}

#[test]
fn shell_shadow_uses_a_dark_theme_token() {
    let css = stylesheet();
    assert!(css.contains("--shadow: #0002"));
    assert!(css.contains("--shadow: #0005"));
    assert!(css.contains("box-shadow:0 8px 24px var(--shadow)"));
}

#[test]
fn focus_remains_visible() {
    let css = stylesheet();
    assert!(css.contains(":focus-visible"));
    assert!(css.contains("outline:3px solid var(--focus)"));
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
fn workspace_and_readable_text_have_bounded_widths() {
    let css = stylesheet();
    assert!(
        css.contains("main { width:100%;") && css.contains("max-width:1024px"),
        "the workspace must stay bounded on wide screens"
    );
    assert!(
        css.contains("max-width:68ch"),
        "long text must keep a readable line length"
    );
}

/// The declarations of the first rule whose selector list names `selector`.
fn declarations_of<'a>(css: &'a str, selector: &str) -> Option<&'a str> {
    css.split('}').find_map(|block| {
        let (selectors, declarations) = block.split_once('{')?;
        selectors
            .split(',')
            .map(str::trim)
            .any(|listed| listed == selector)
            .then_some(declarations)
    })
}

#[test]
fn body_text_wraps_long_words_inside_the_screen() {
    let css = stylesheet();
    let body = declarations_of(&css, "body").unwrap_or_default();
    assert!(
        body.contains("overflow-wrap: anywhere"),
        "long words must wrap instead of overflowing a phone screen"
    );
}

#[test]
fn images_scale_down_to_their_container() {
    let css = stylesheet();
    let image = declarations_of(&css, "img").unwrap_or_default();
    assert!(
        image.contains("max-width: 100%"),
        "images must not be wider than their container"
    );
}

#[test]
fn import_method_disclosure_has_one_rotating_marker_and_divider() {
    let css = stylesheet();
    let marker = declarations_of(&css, ".import-method summary::before").unwrap_or_default();
    let open_marker =
        declarations_of(&css, ".import-method[open] > summary::before").unwrap_or_default();
    let method = declarations_of(&css, ".import-method").unwrap_or_default();
    let catalogue_note = declarations_of(&css, ".import-catalog-note").unwrap_or_default();
    assert!(marker.contains("content:\"▶\""));
    assert!(open_marker.contains("transform:rotate(90deg)"));
    assert!(method.contains("border-bottom:1px solid var(--border)"));
    assert!(!catalogue_note.contains("border-top:"));
}

#[test]
fn no_fixed_width_is_wider_than_a_phone_screen() {
    let css = stylesheet();
    for declaration in css.split([';', '{', '}']).map(str::trim) {
        let Some(value) = declaration.strip_prefix("width:") else {
            continue;
        };
        if let Some(pixels) = value.trim().strip_suffix("px") {
            let pixels: Option<f64> = pixels.trim().parse().ok();
            assert!(
                pixels.is_some_and(|px| px <= 360.0),
                "a fixed width wider than 360px overflows a phone: {declaration}"
            );
        }
    }
}
