//! Visible text comes from the locale files. A string literal used as a text child inside a
//! `view!` block is hard-coded text and fails the check. Attribute values and the arguments of
//! calls such as `t("key")` are allowed.

#![expect(
    clippy::indexing_slicing,
    clippy::string_slice,
    clippy::unwrap_used,
    reason = "the scanner only indexes offsets it has checked, and the test stops on read failures"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// The index of the quote that closes the string literal opened at `start`, or the length of
/// `bytes` when the literal is not closed.
const fn literal_end(bytes: &[u8], start: usize) -> usize {
    let mut index = start + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'"' => return index,
            _ => index += 1,
        }
    }
    bytes.len()
}

/// The index of the bracket that closes the one at `open`, or the length of `bytes` when it is
/// not closed. String literals are skipped.
const fn closing_bracket(bytes: &[u8], open: usize) -> usize {
    let mut depth = 0_usize;
    let mut index = open;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => index = literal_end(bytes, index),
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return index;
                }
            }
            _ => {}
        }
        index += 1;
    }
    bytes.len()
}

/// The string literals in the body of one `view!` block that are used as text: not directly after
/// `=`, and not inside parentheses.
fn text_literals(body: &str) -> Vec<String> {
    let bytes = body.as_bytes();
    let mut found = Vec::new();
    let mut parens = 0_usize;
    let mut previous = b' ';
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'"' {
            let end = literal_end(bytes, index).min(bytes.len());
            if parens == 0 && previous != b'=' {
                found.push(body[index + 1..end].to_owned());
            }
            previous = b'"';
            index = end + 1;
            continue;
        }
        match byte {
            b'(' => parens += 1,
            b')' => parens = parens.saturating_sub(1),
            _ => {}
        }
        if !byte.is_ascii_whitespace() {
            previous = byte;
        }
        index += 1;
    }
    found
}

/// The string literals used as text in every `view!` block of `source`.
fn text_children(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut found = Vec::new();
    let mut cursor = 0;
    while let Some(offset) = source[cursor..].find("view!") {
        let at = cursor + offset;
        cursor = at + "view!".len();
        if at > 0 && (bytes[at - 1].is_ascii_alphanumeric() || bytes[at - 1] == b'_') {
            continue;
        }
        let mut open = cursor;
        while bytes.get(open).is_some_and(u8::is_ascii_whitespace) {
            open += 1;
        }
        if !matches!(bytes.get(open), Some(b'{' | b'(' | b'[')) {
            continue;
        }
        let close = closing_bracket(bytes, open);
        found.extend(text_literals(&source[open + 1..close]));
        cursor = close;
    }
    found
}

fn rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

#[test]
fn flags_a_string_used_as_a_text_child() {
    let source = r#"view! { <h1>"crabbage"</h1> }"#;
    assert_eq!(text_children(source), ["crabbage"]);
}

#[test]
fn flags_a_text_child_next_to_a_translated_one() {
    let source = r#"view! { <p>{t("app.title")} "tail"</p> }"#;
    assert_eq!(text_children(source), ["tail"]);
}

#[test]
fn allows_attribute_values_and_call_arguments() {
    let source = r#"view! { <Title text="crabbage"/><form method="post" action="/auth/logout"><button>{t("app.sign_out")}</button></form> }"#;
    assert_eq!(text_children(source), Vec::<&str>::new());
}

#[test]
fn ignores_strings_outside_views_and_macros_named_like_view() {
    let source = r#"fn label() -> &'static str { "free" } preview! { "not a view" }"#;
    assert_eq!(text_children(source), Vec::<&str>::new());
}

#[test]
fn no_source_file_has_hard_coded_text_in_a_view() {
    let mut files = Vec::new();
    rust_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    assert_ne!(files, Vec::<PathBuf>::new());
    let mut violations = Vec::new();
    for file in files {
        let source = fs::read_to_string(&file).unwrap();
        for literal in text_children(&source) {
            violations.push(format!("{}: \"{literal}\"", file.display()));
        }
    }
    assert!(
        violations.is_empty(),
        "hard-coded text in view! blocks, move it to locales/:\n{}",
        violations.join("\n")
    );
}
