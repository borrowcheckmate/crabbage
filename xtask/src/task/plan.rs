use crate::cli::Command;

use super::action::Action;
use super::step::Step;

const IMAGE: &str = "crabbage:local";
const COVERAGE_THRESHOLD: &str = "90";
const SSR_FEATURE: &str = "crabbage/ssr";

pub(crate) fn plan(command: &Command) -> Vec<Action> {
    match command {
        Command::Fmt { check } => vec![fmt(*check)],
        Command::Lint => vec![lint()],
        Command::Test => vec![test()],
        Command::Docs => vec![docs()],
        Command::Coverage => vec![coverage()],
        Command::Deny => vec![deny()],
        Command::Policy => vec![Action::Policy],
        Command::Identity { range } => vec![Action::Identity(range.clone())],
        Command::CommitMsg { file } => vec![Action::CommitMsg(file.clone())],
        Command::PrCheck => vec![Action::PrCheck],
        Command::Docker => vec![docker()],
        Command::Wasm => vec![wasm()],
        Command::PreCommit => vec![
            Action::Identity(None),
            Action::Policy,
            fmt(true),
            lint(),
            test(),
        ],
        Command::Ci => vec![
            fmt(true),
            Action::Policy,
            lint(),
            docs(),
            test(),
            deny(),
            wasm(),
        ],
        Command::Hooks => vec![Action::Run(Step::new(
            "git",
            &["config", "core.hooksPath", ".githooks"],
        ))],
    }
}

fn fmt(check: bool) -> Action {
    let args: &[&str] = if check {
        &["fmt", "--all", "--check"]
    } else {
        &["fmt", "--all"]
    };
    Action::Run(Step::cargo(args))
}

fn lint() -> Action {
    Action::Run(Step::cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--features",
        SSR_FEATURE,
        "--locked",
        "--",
        "-D",
        "warnings",
    ]))
}

fn test() -> Action {
    Action::Run(Step::cargo(&[
        "test",
        "--workspace",
        "--features",
        SSR_FEATURE,
        "--locked",
    ]))
}

fn docs() -> Action {
    Action::Run(
        Step::cargo(&[
            "doc",
            "--workspace",
            "--no-deps",
            "--document-private-items",
            "--locked",
        ])
        .with_env("RUSTDOCFLAGS", "-D warnings"),
    )
}

fn coverage() -> Action {
    Action::Run(Step::cargo(&[
        "llvm-cov",
        "--workspace",
        "--exclude",
        "xtask",
        "--features",
        SSR_FEATURE,
        "--locked",
        "--fail-under-lines",
        COVERAGE_THRESHOLD,
    ]))
}

fn deny() -> Action {
    Action::Run(Step::cargo(&["deny", "--locked", "check"]))
}

fn wasm() -> Action {
    Action::Run(Step::cargo(&[
        "clippy",
        "--package",
        "crabbage",
        "--lib",
        "--target",
        "wasm32-unknown-unknown",
        "--no-default-features",
        "--features",
        "hydrate",
        "--locked",
        "--",
        "-D",
        "warnings",
    ]))
}

fn docker() -> Action {
    Action::Run(Step::new("docker", &["build", "--tag", IMAGE, "."]))
}

#[cfg(test)]
mod tests {
    use super::{Action, Command, plan};

    fn shell_lines(command: &Command) -> Vec<String> {
        plan(command)
            .into_iter()
            .map(|action| match action {
                Action::Run(step) => step.to_string(),
                Action::Policy | Action::Identity(_) | Action::CommitMsg(_) | Action::PrCheck => {
                    format!("{action:?}")
                }
            })
            .collect()
    }

    #[test]
    fn fmt_check_does_not_rewrite() {
        assert_eq!(
            shell_lines(&Command::Fmt { check: true }),
            ["cargo fmt --all --check"]
        );
        assert_eq!(
            shell_lines(&Command::Fmt { check: false }),
            ["cargo fmt --all"]
        );
    }

    #[test]
    fn lint_denies_warnings() {
        let lines = shell_lines(&Command::Lint);
        assert!(lines[0].ends_with("-- -D warnings"), "{lines:?}");
    }

    #[test]
    fn pre_commit_checks_identity_and_leaks_first() {
        let actions = plan(&Command::PreCommit);
        assert_eq!(actions[0], Action::Identity(None));
        assert_eq!(actions[1], Action::Policy);
        assert_eq!(actions.len(), 5);
    }

    #[test]
    fn ci_covers_format_policy_lint_docs_tests_and_dependencies() {
        let lines = shell_lines(&Command::Ci);
        assert_eq!(lines.len(), 7);
        assert!(lines.contains(&"Policy".to_owned()));
        for needle in [
            "fmt",
            "clippy",
            "cargo doc",
            "cargo test",
            "cargo deny",
            "wasm32",
        ] {
            assert!(lines.iter().any(|line| line.contains(needle)), "{needle}");
        }
    }

    #[test]
    fn coverage_has_a_threshold() {
        assert!(shell_lines(&Command::Coverage)[0].contains("--fail-under-lines 90"));
    }
}
