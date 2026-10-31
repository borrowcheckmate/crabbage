//! Whole-word matching for the words an ingredient line is read with.

/// The text after `word` when `word` is a whole word at the start of `text`. Case is ignored.
pub(crate) fn strip<'a>(text: &'a str, word: &str) -> Option<&'a str> {
    let head = text.get(..word.len())?;
    let rest = text.get(word.len()..)?;
    (head.to_lowercase() == word && !rest.starts_with(char::is_alphabetic)).then_some(rest)
}
