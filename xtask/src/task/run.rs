use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command as Process;

use crate::cli::Command;
use crate::policy::{self, Violation};
use crate::repo;

use super::action::Action;
use super::plan::plan;
use super::smoke;
use super::step::Step;

pub(crate) fn run(command: &Command) -> Result<(), String> {
    let root = repo::root()?;
    plan(command)
        .iter()
        .try_for_each(|action| execute(action, &root))
}

fn execute(action: &Action, root: &Path) -> Result<(), String> {
    match action {
        Action::Run(step) => run_step(step, root),
        Action::Smoke => smoke(root),
        Action::Policy => report(&policy::check_repository(root, &repo::tracked_files(root)?)),
        Action::Identity(None) => report(&policy::check_identities(
            &repo::current_identity_emails(root)?,
        )),
        Action::Identity(Some(range)) => report(&policy::check_identities(&repo::commit_emails(
            root, range,
        )?)),
        Action::CommitMsg(file) => {
            let message =
                fs::read_to_string(file).map_err(|error| format!("{}: {error}", file.display()))?;
            report(&policy::check_commit_message(&message))
        }
        Action::PrCheck => {
            let title = env::var("PR_TITLE").map_err(|error| format!("PR_TITLE: {error}"))?;
            let body = env::var("PR_BODY").unwrap_or_default();
            report(&policy::check_pull_request(&title, &body))
        }
    }
}

fn report(violations: &[Violation]) -> Result<(), String> {
    let mut stderr = io::stderr().lock();
    for violation in violations {
        writeln!(stderr, "{violation}").map_err(|error| error.to_string())?;
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!("{} policy violation(s)", violations.len()))
    }
}

fn smoke(root: &Path) -> Result<(), String> {
    let port = smoke::free_port()?;
    let started = run_step(&smoke::start_step(port), root);
    let checked = started.and_then(|()| smoke::wait_for_healthz(port));
    let removed = run_step(&smoke::remove_step(), root);
    checked.and(removed)
}

fn run_step(step: &Step, root: &Path) -> Result<(), String> {
    writeln!(io::stderr(), "$ {step}").map_err(|error| error.to_string())?;
    let mut process = Process::new(&step.program);
    process.args(&step.args).current_dir(root);
    for (key, value) in &step.env {
        process.env(key, value);
    }
    let status = process
        .status()
        .map_err(|error| format!("cannot run {}: {error}", step.program))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`{step}` failed with {status}"))
    }
}
