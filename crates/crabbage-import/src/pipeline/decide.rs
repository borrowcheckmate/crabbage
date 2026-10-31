use super::Imported;

/// Whether the AI pass is needed.
///
/// It is needed when the title is blank, the servings, lines or steps are missing, a line matches no
/// ingredient, or the page is in another language than the instance's. A page without a language
/// does not count as another language.
#[must_use]
pub fn needs_ai(imported: &Imported, page_language: Option<&str>, instance_language: &str) -> bool {
    imported.title.trim().is_empty()
        || imported.servings.is_none()
        || imported.lines.is_empty()
        || imported.steps.is_empty()
        || !imported.unmatched.is_empty()
        || page_language.is_some_and(|language| primary(language) != primary(instance_language))
}

/// The primary language subtag, lowercased: `nl` for `nl-NL` and `nl_BE`.
fn primary(language: &str) -> String {
    language
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .to_lowercase()
}
