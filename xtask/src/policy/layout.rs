pub(crate) const MAX_FILE_BYTES: usize = 256 * 1024;

const ROOT_DOCUMENTS: [&str; 6] = [
    "README.md",
    "AGENTS.md",
    "ARCHITECTURE.md",
    "CONTRIBUTING.md",
    "SECURITY.md",
    "CHANGELOG.md",
];
const DOCUMENT_EXTENSIONS: [&str; 11] = [
    "md", "markdown", "txt", "rst", "adoc", "org", "pdf", "doc", "docx", "odt", "rtf",
];
const SECRET_NAMES: [&str; 9] = [
    ".netrc",
    ".npmrc",
    ".pypirc",
    "credentials",
    "credentials.json",
    "id_dsa",
    "id_ecdsa",
    "id_ed25519",
    "id_rsa",
];
const SECRET_EXTENSIONS: [&str; 10] = [
    "pem", "key", "p12", "pfx", "jks", "keystore", "ppk", "kdbx", "asc", "gpg",
];
const METADATA_NAMES: [&str; 3] = [".DS_Store", "Thumbs.db", "desktop.ini"];
const METADATA_EXTENSIONS: [&str; 6] = ["xls", "xlsx", "ppt", "pptx", "ods", "odp"];

pub(crate) fn layout_problem(path: &str) -> Option<&'static str> {
    let name = path.rsplit('/').next().unwrap_or(path);
    let extension = name
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .unwrap_or_default();
    let extension = extension.as_str();
    if name == ".env" || name.starts_with(".env.") {
        Some("environment files must never be committed")
    } else if SECRET_NAMES.contains(&name) || SECRET_EXTENSIONS.contains(&extension) {
        Some("looks like a credential or key file")
    } else if METADATA_NAMES.contains(&name) || METADATA_EXTENSIONS.contains(&extension) {
        Some("file type carries personal metadata")
    } else if DOCUMENT_EXTENSIONS.contains(&extension) && !is_allowed_document(path) {
        Some(
            "loose document; use README.md, AGENTS.md, ARCHITECTURE.md, CONTRIBUTING.md, \
             SECURITY.md or an issue",
        )
    } else {
        None
    }
}

pub(crate) const fn is_blob(size: usize) -> bool {
    size > MAX_FILE_BYTES
}

fn is_allowed_document(path: &str) -> bool {
    let parts: Vec<&str> = path.split('/').collect();
    ROOT_DOCUMENTS.contains(&path)
        || path.starts_with(".github/")
        || matches!(parts.as_slice(), ["crates", _, "CHANGELOG.md"])
}

#[cfg(test)]
mod tests {
    use super::{MAX_FILE_BYTES, is_blob, layout_problem};

    #[test]
    fn accepts_regular_files() {
        for path in [
            "README.md",
            "crates/app/CHANGELOG.md",
            ".github/PULL_REQUEST_TEMPLATE.md",
            "LICENSE",
            "src/lib.rs",
            ".gitignore",
        ] {
            assert_eq!(layout_problem(path), None, "{path}");
        }
    }

    #[test]
    fn rejects_environment_files() {
        for path in [".env", "app/.env.local", ".env.example"] {
            assert!(
                layout_problem(path).unwrap().contains("environment"),
                "{path}"
            );
        }
    }

    #[test]
    fn rejects_credentials() {
        for path in ["certs/server.PEM", "id_ed25519", "deploy/.netrc"] {
            assert!(
                layout_problem(path).unwrap().contains("credential"),
                "{path}"
            );
        }
    }

    #[test]
    fn rejects_metadata_carriers() {
        for path in ["docs/.DS_Store", "plan.xlsx"] {
            assert!(layout_problem(path).unwrap().contains("metadata"), "{path}");
        }
    }

    #[test]
    fn rejects_loose_documents() {
        for path in [
            "NOTES.md",
            "docs/design.md",
            "crates/app/README.md",
            "todo.txt",
        ] {
            assert!(layout_problem(path).unwrap().contains("loose"), "{path}");
        }
    }

    #[test]
    fn flags_large_files() {
        assert!(!is_blob(MAX_FILE_BYTES));
        assert!(is_blob(MAX_FILE_BYTES + 1));
    }
}
