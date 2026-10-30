const ALLOWED_STATEMENTS: [&str; 7] = [
    "mod ",
    "pub mod ",
    "pub(crate) mod ",
    "pub(super) mod ",
    "pub use ",
    "pub(crate) use ",
    "pub(super) use ",
];
const SNIPPET_CHARS: usize = 60;

pub(crate) fn is_module_root(path: &str) -> bool {
    path.rsplit('/').next() == Some("mod.rs")
}

pub(crate) fn foreign_statements(source: &str) -> Vec<String> {
    let code = source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//") && !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join(" ");
    code.split(';')
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
        .filter(|statement| !ALLOWED_STATEMENTS.iter().any(|p| statement.starts_with(p)))
        .map(|statement| statement.chars().take(SNIPPET_CHARS).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{foreign_statements, is_module_root};

    #[test]
    fn recognises_module_roots() {
        assert!(is_module_root("crates/a/src/greeting/mod.rs"));
        assert!(!is_module_root("crates/a/src/lib.rs"));
        assert!(!is_module_root("crates/a/src/main.rs"));
        assert!(!is_module_root("crates/a/src/model.rs"));
    }

    #[test]
    fn accepts_declarations_and_reexports() {
        let source = "//! Overview.\n\n#[cfg(test)]\nmod tests;\npub mod api;\nmod inner;\n\
                      pub use inner::{\n    A,\n    B,\n};\npub(crate) use inner::C;\n";
        assert_eq!(foreign_statements(source), Vec::<String>::new());
    }

    #[test]
    fn rejects_code() {
        let source = "mod inner;\nuse std::fmt;\npub fn helper() {}\n";
        assert_eq!(
            foreign_statements(source),
            ["use std::fmt", "pub fn helper() {}"]
        );
    }
}
