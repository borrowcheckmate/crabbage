use std::fs;
use std::path::Path;

use super::Violation;
use super::comments::plain_comment_lines;
use super::layout::{MAX_FILE_BYTES, is_blob, layout_problem};
use super::leaks::{declares_authors, find_leaks, has_image_metadata};
use super::length::{MAX_LINES, excess_lines};
use super::module_root::{foreign_statements, is_module_root};

pub(crate) fn check_repository(root: &Path, files: &[String]) -> Vec<Violation> {
    files
        .iter()
        .filter_map(|path| {
            fs::read(root.join(path))
                .ok()
                .map(|bytes| check_file(path, &bytes))
        })
        .flatten()
        .collect()
}

fn check_file(path: &str, bytes: &[u8]) -> Vec<Violation> {
    let mut violations = Vec::new();
    if let Some(problem) = layout_problem(path) {
        violations.push(Violation::new(path, problem));
    }
    if is_blob(bytes.len()) {
        violations.push(Violation::new(
            path,
            format!(
                "{} bytes exceeds the {MAX_FILE_BYTES} byte limit; generate it instead",
                bytes.len()
            ),
        ));
    }
    if has_image_metadata(path, bytes) {
        violations.push(Violation::new(path, "image contains EXIF or XMP metadata"));
    }
    let text = String::from_utf8_lossy(bytes);
    violations.extend(
        find_leaks(&text)
            .into_iter()
            .map(|(line, message)| Violation::at_line(path, line, message)),
    );
    if declares_authors(path, &text) {
        violations.push(Violation::new(
            path,
            "remove `authors`; it leaks personal data",
        ));
    }
    if Path::new(path)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("rs"))
    {
        violations.extend(check_rust(path, &text));
    }
    violations
}

fn check_rust(path: &str, source: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    if let Some(lines) = excess_lines(source) {
        violations.push(Violation::new(
            path,
            format!("{lines} lines exceeds {MAX_LINES}; split it into submodules"),
        ));
    }
    violations.extend(plain_comment_lines(source).into_iter().map(|line| {
        Violation::at_line(
            path,
            line,
            "plain comment; make the code self-explanatory, use `///`, or `// NOTE:` if unavoidable",
        )
    }));
    if is_module_root(path) {
        violations.extend(foreign_statements(source).into_iter().map(|statement| {
            Violation::new(
                path,
                format!("module roots may only declare and re-export modules: `{statement}`"),
            )
        }));
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::check_file;

    fn messages(path: &str, content: &str) -> Vec<String> {
        check_file(path, content.as_bytes())
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    #[test]
    fn accepts_clean_sources() {
        let library = messages("src/lib.rs", "//! Docs.\n\nmod a;\n\npub use a::A;\n");
        assert_eq!(library, Vec::<String>::new());
        let module = messages("src/a.rs", "/// Docs.\npub struct A;\n");
        assert_eq!(module, Vec::<String>::new());
    }

    #[test]
    fn reports_every_rust_rule() {
        let found = messages(
            "src/greeting/mod.rs",
            "// overview\nmod a;\nfn helper() {}\n",
        );
        assert_eq!(found.len(), 2, "{found:?}");
        assert!(found[0].starts_with("src/greeting/mod.rs:1: plain comment"));
        assert!(found[1].contains("`fn helper() {}`"));
        let long = "fn a() {}\n".repeat(501);
        assert!(messages("src/a.rs", &long)[0].contains("501 lines"));
    }

    #[test]
    fn reports_layout_size_metadata_and_leaks() {
        assert!(messages(".env", "KEY=1")[0].contains("environment"));
        assert!(messages("big.bin", &"x".repeat(300 * 1024))[0].contains("byte limit"));
        assert!(messages("a.jpg", "..Exif..")[0].contains("EXIF"));
        let path = ["", "home", "jdoe", "x"].join("/");
        assert!(messages("build.sh", &path)[0].contains("build.sh:1: path into a home"));
        assert!(messages("Cargo.toml", "authors = []")[0].contains("authors"));
    }

    #[test]
    fn checks_non_utf8_files() {
        let mut bytes = vec![0xff, 0xfe];
        bytes.extend_from_slice(["", "Users", "jdoe", "x"].join("/").as_bytes());
        assert_eq!(check_file("blob.dat", &bytes).len(), 1);
    }
}
