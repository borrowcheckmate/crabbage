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
fn desktop_top_bar_and_side_navigation_stay_in_view() {
    let css = stylesheet();
    let (_, desktop) = css
        .split_once("@media (min-width:900px) and (min-height:501px) {")
        .unwrap();
    let topbar = declarations_of(desktop, ".topbar").unwrap_or_default();
    let navigation = declarations_of(desktop, ".primary-nav").unwrap_or_default();
    assert!(topbar.contains("position:sticky") && topbar.contains("top:0"));
    assert!(topbar.contains("height:var(--topbar-height)"));
    assert!(navigation.contains("position:sticky"));
    assert!(navigation.contains("top:var(--topbar-height)"));
    assert!(navigation.contains("height:calc(100dvh - var(--topbar-height))"));
    assert!(navigation.contains("overflow-y:auto"));
    for panel in [".shopping-review", ".plan-editor", ".import-source-desktop"] {
        let declarations = css
            .split('}')
            .filter_map(|block| block.split_once('{'))
            .find(|(selector, body)| {
                selector.trim_end().ends_with(panel) && body.contains("sticky")
            })
            .map(|(_, body)| body)
            .unwrap_or_default();
        assert!(
            declarations.contains("var(--topbar-height, 0px)"),
            "{panel} must stick below the top bar"
        );
    }
}

#[test]
fn recipe_compact_header_appears_once_the_title_scrolls_away() {
    let css = stylesheet();
    assert!(
        declarations_of(&css, ".recipe-compact").is_some_and(|rule| rule.contains("display:none"))
    );
    let (_, supported) = css
        .split_once("@supports (animation-timeline: view()) {")
        .unwrap();
    let hero = declarations_of(supported, ".recipe-detail-header").unwrap_or_default();
    let compact = declarations_of(supported, ".recipe-compact").unwrap_or_default();
    assert!(hero.contains("view-timeline-name:--recipe-hero"));
    assert!(hero.contains("view-timeline-inset:var(--topbar-height, 0px)"));
    assert!(compact.contains("position:sticky"));
    assert!(compact.contains("top:var(--topbar-height, 0px)"));
    let bleed = declarations_of(supported, ".recipe-compact::before").unwrap_or_default();
    assert!(
        bleed.contains("inset:0 -100vw") && bleed.contains("border-bottom:1px solid var(--border)")
    );
    let shell = declarations_of(supported, ".shell-layout").unwrap_or_default();
    assert!(
        shell.contains("overflow-x:clip"),
        "the full-bleed bar must not scroll sideways"
    );
    assert!(compact.contains("visibility:hidden"));
    assert!(compact.contains("animation-timeline:--recipe-hero"));
    assert!(css.contains("@keyframes recipe-compact-reveal"));
    let back = declarations_of(&css, ".recipe-compact-back").unwrap_or_default();
    assert!(back.contains("min-height:44px") && back.contains("min-width:44px"));
}

#[test]
fn recipe_pages_fill_the_workspace_like_today() {
    let css = stylesheet();
    for selector in [
        ".recipe-list-heading",
        ".recipe-list-content",
        ".recipe-detail",
    ] {
        let capped = css
            .split('}')
            .filter_map(|block| block.split_once('{'))
            .any(|(selectors, body)| {
                selectors
                    .split(',')
                    .map(str::trim)
                    .any(|listed| listed == selector)
                    && body.contains("max-width")
            });
        assert!(
            !capped,
            "{selector} must share the workspace edges with the other pages"
        );
    }
    let grid = declarations_of(&css, ".recipe-grid").unwrap_or_default();
    assert!(
        grid.contains("repeat(auto-fill,minmax(min(16rem,100%),1fr))"),
        "the recipe cards fill the column with as many 16rem cards as fit: {grid}"
    );
}

#[test]
fn pages_share_one_centred_column_in_the_content_area() {
    let css = stylesheet();
    let workspace = declarations_of(&css, "main").unwrap_or_default();
    assert!(workspace.contains("max-width:1024px"), "{workspace}");
    assert!(workspace.contains("margin:0 auto"), "{workspace}");
    assert!(
        css.contains(".admin-page { width:100%; max-width:52rem; margin:0 auto;"),
        "administration pages must be centred in the column"
    );
    assert!(
        css.contains("max-width:42rem; margin-inline:auto;"),
        "the state card must be centred in the column too"
    );
}

#[test]
fn today_day_card_states_the_day_and_aligns_with_the_side_column() {
    let css = stylesheet();
    let day = declarations_of(&css, ".today-day h2").unwrap_or_default();
    assert!(
        day.contains("1.55rem"),
        "the day state is heading-sized: {day}"
    );
    let heading = declarations_of(&css, ".today-side .today-section-heading").unwrap_or_default();
    assert!(heading.contains("margin-top: 0"));
    assert!(
        heading.contains("align-items: flex-start"),
        "the Coming up heading starts at the top of the day card: {heading}"
    );
}

#[test]
fn today_title_link_is_a_44_pixel_target_without_moving_the_heading() {
    let css = stylesheet();
    let link = declarations_of(&css, ".today-feature-copy h2 a").unwrap_or_default();
    assert!(link.contains("display: block"), "{link}");
    assert!(
        link.contains("padding-block: calc((44px - 1.2em) / 2)"),
        "{link}"
    );
    assert!(
        link.contains("margin-block: calc((1.2em - 44px) / 2)"),
        "{link}"
    );
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
fn import_review_amounts_have_a_fixed_column_and_a_hidden_missing_text() {
    let css = stylesheet();
    let amount = declarations_of(&css, ".import-amount").unwrap_or_default();
    let missing = declarations_of(&css, ".import-amount-missing").unwrap_or_default();
    assert!(amount.contains("flex:0 0 4.5rem"));
    assert!(missing.contains("clip:rect(0,0,0,0)"));
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
