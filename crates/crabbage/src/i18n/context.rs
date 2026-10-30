use leptos::prelude::use_context;

use super::{Language, Messages};

/// The language of the current render, from the context the server or the hydration provides.
/// English when none is provided.
pub(crate) fn current_language() -> Language {
    use_context::<Language>().unwrap_or(Language::En)
}

/// The messages of the current language.
pub(crate) fn messages() -> &'static Messages {
    Messages::for_language(current_language())
}

/// The text of the message `key` in the current language.
pub(crate) fn t(key: &'static str) -> &'static str {
    messages().get(key)
}

/// The text of the message `key` in the current language, with each `{name}` replaced by the value
/// paired with `name` in `args`.
pub(crate) fn t_with(key: &str, args: &[(&str, &str)]) -> String {
    messages().format(key, args)
}

/// The language the server rendered the page in, from the `lang` attribute of the document
/// element. English when the attribute is missing or unknown.
#[cfg(feature = "hydrate")]
pub(crate) fn document_language() -> Language {
    leptos::prelude::document()
        .document_element()
        .and_then(|element| element.get_attribute("lang"))
        .and_then(|code| Language::from_code(&code))
        .unwrap_or(Language::En)
}
