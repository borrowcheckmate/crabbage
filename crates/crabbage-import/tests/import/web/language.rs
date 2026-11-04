use crabbage_import::web::extract;

fn page(html_tag: &str, recipe_extra: &str) -> String {
    let json = format!(r#"{{"@type": "Recipe", "name": "Test"{recipe_extra}}}"#);
    format!(
        "<!doctype html>{html_tag}<head><script type=\"application/ld+json\">{json}</script></head><body></body></html>"
    )
}

#[test]
fn the_recipe_in_language_is_read_before_the_page_language() {
    let draft =
        extract(&page(r#"<html lang="nl">"#, r#", "inLanguage": "en-GB""#)).expect("recipe");
    assert_eq!(draft.language.as_deref(), Some("en-GB"));
}

#[test]
fn the_html_lang_is_read_when_the_recipe_has_no_language() {
    let draft = extract(&page(r#"<html lang="nl-NL">"#, "")).expect("recipe");
    assert_eq!(draft.language.as_deref(), Some("nl-NL"));
}

#[test]
fn the_html_lang_accepts_single_quotes_other_attributes_and_any_case() {
    let single = extract(&page("<HTML DIR='ltr' LANG='de'>", "")).expect("recipe");
    assert_eq!(single.language.as_deref(), Some("de"));
    let bare = extract(&page("<html class=x lang=fr>", "")).expect("recipe");
    assert_eq!(bare.language.as_deref(), Some("fr"));
}

#[test]
fn a_page_with_no_language_has_none() {
    assert_eq!(extract(&page("<html>", "")).expect("recipe").language, None);
    assert_eq!(
        extract(&page("<html xml:lang=\"en\">", ""))
            .expect("recipe")
            .language,
        None
    );
    assert_eq!(
        extract(&page("<html lang=\"\">", ""))
            .expect("recipe")
            .language,
        None
    );
}
