use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "cargo xtask", about = "Repository automation")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum Command {
    /// Format all code, or only verify formatting with --check.
    Fmt {
        #[arg(long)]
        check: bool,
    },
    /// Run clippy on every target with warnings denied.
    Lint,
    /// Run every test in the workspace.
    Test,
    /// Build documentation with warnings denied.
    Docs,
    /// Enforce the line coverage threshold (needs cargo-llvm-cov).
    Coverage,
    /// Check advisories, licenses, bans and sources (needs cargo-deny).
    Deny,
    /// Check repository rules: layout, file length, comments, mod.rs, leaks.
    Policy,
    /// Verify author and committer emails are anonymous.
    Identity {
        /// Check every commit in a git revision range instead of the current identity.
        #[arg(long)]
        range: Option<String>,
    },
    /// Validate a commit message file. Used by the commit-msg hook.
    CommitMsg { file: PathBuf },
    /// Validate a pull request from the `PR_TITLE` and `PR_BODY` environment variables.
    PrCheck,
    /// Build the container image and smoke-test `/healthz` in a hardened container.
    Docker,
    /// Lint the WASM build of the app with warnings denied.
    Wasm,
    /// Run the checks of the pre-commit hook.
    PreCommit,
    /// Run every check CI runs, except coverage and docker.
    Ci,
    /// Use the tracked hooks in .githooks for this clone.
    Hooks,
    /// Build and serve the app with generated demo data.
    Demo {
        #[arg(long, default_value_t = 3000)]
        port: u16,
        #[arg(long)]
        keep: bool,
        #[arg(long, default_value = "en")]
        language: String,
        #[arg(long)]
        ai: Option<String>,
    },
    /// Run hydrated browser checks and save screenshots under target/browser-shots.
    Browser,
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};

    use super::{Cli, Command};

    #[test]
    fn definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_flags_and_arguments() {
        let cli = Cli::try_parse_from(["xtask", "fmt", "--check"]).unwrap();
        assert_eq!(cli.command, Command::Fmt { check: true });
        let cli = Cli::try_parse_from(["xtask", "identity", "--range", "a..b"]).unwrap();
        assert_eq!(
            cli.command,
            Command::Identity {
                range: Some("a..b".to_owned())
            }
        );
        let cli = Cli::try_parse_from([
            "xtask",
            "demo",
            "--port",
            "3200",
            "--keep",
            "--language",
            "nl",
            "--ai",
            "http://127.0.0.1:8080/v1",
        ])
        .unwrap();
        assert_eq!(
            cli.command,
            Command::Demo {
                port: 3200,
                keep: true,
                language: "nl".to_owned(),
                ai: Some("http://127.0.0.1:8080/v1".to_owned()),
            }
        );
        assert_eq!(
            Cli::try_parse_from(["xtask", "browser"]).unwrap().command,
            Command::Browser
        );
    }
}
