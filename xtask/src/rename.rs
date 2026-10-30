use std::fs;
use std::path::Path;

use crate::repo;

const CRATE_NAME: &str = "rust-template";

pub(crate) fn rename(root: &Path, name: &str) -> Result<(), String> {
    validate(name)?;
    for path in repo::tracked_files(root)? {
        let file = root.join(&path);
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        let renamed = replace_identifiers(&text, name);
        if renamed != text {
            fs::write(&file, renamed).map_err(|error| format!("{path}: {error}"))?;
        }
    }
    let crates = root.join("crates");
    fs::rename(crates.join(CRATE_NAME), crates.join(name))
        .map_err(|error| format!("cannot move crates/{CRATE_NAME}: {error}"))
}

fn validate(name: &str) -> Result<(), String> {
    let starts_with_letter = name.chars().next().is_some_and(|c| c.is_ascii_lowercase());
    let allowed_chars = name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if starts_with_letter && allowed_chars && !name.ends_with('-') && name != CRATE_NAME {
        Ok(())
    } else {
        Err(format!(
            "`{name}` must be a new kebab-case name: lowercase letters, digits and dashes"
        ))
    }
}

fn replace_identifiers(text: &str, name: &str) -> String {
    text.replace(CRATE_NAME, name)
        .replace(&CRATE_NAME.replace('-', "_"), &name.replace('-', "_"))
}

#[cfg(test)]
mod tests {
    use super::{CRATE_NAME, replace_identifiers, validate};

    #[test]
    fn accepts_kebab_case_names() {
        assert!(validate(&format!("{CRATE_NAME}-next")).is_ok());
        assert!(validate(&format!("{CRATE_NAME}2")).is_ok());
    }

    #[test]
    fn rejects_invalid_names() {
        for name in ["", "My-App", "1app", "my_app", "app-", CRATE_NAME] {
            assert!(validate(name).is_err(), "{name}");
        }
    }

    #[test]
    fn replaces_package_and_library_names() {
        let library = CRATE_NAME.replace('-', "_");
        let text = format!("name = \"{CRATE_NAME}\"\nuse {library}::Name;\n");
        assert_eq!(
            replace_identifiers(&text, "my-app"),
            "name = \"my-app\"\nuse my_app::Name;\n"
        );
    }
}
