use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command as Process;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cli::Command;
use crate::policy::{self, Violation};
use crate::repo;

use super::action::Action;
use super::plan::plan;
use super::smoke;
use super::step::Step;

pub(crate) fn run(command: &Command) -> Result<(), String> {
    let root = repo::root()?;
    if let Command::Demo {
        port,
        keep,
        language,
        ai,
    } = command
    {
        return demo(&root, *port, *keep, language, ai.as_deref());
    }
    plan(command)
        .iter()
        .try_for_each(|action| execute(action, &root))
}

fn demo(
    root: &Path,
    port: u16,
    keep: bool,
    language: &str,
    ai: Option<&str>,
) -> Result<(), String> {
    if !matches!(language, "en" | "nl") {
        return Err("--language must be en or nl".to_owned());
    }
    let data_dir = if keep {
        root.join("target/demo-data")
    } else {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        env::temp_dir().join(format!("crabbage-demo-{}-{nonce}", std::process::id()))
    };
    let runtime = tokio::runtime::Runtime::new().map_err(|error| error.to_string())?;
    runtime.block_on(async {
        let issuer = crabbage_dev::FakeIssuer::start().await;
        crabbage_dev::load_demo(&data_dir, language)
            .await
            .map_err(|error| error.to_string())?;
        if let Some(base_url) = &ai {
            crabbage_dev::enable_demo_ai(&data_dir, base_url)
                .await
                .map_err(|error| error.to_string())?;
        }
        let mut child = Process::new("cargo");
        child
            .args(["leptos", "serve"])
            .current_dir(root)
            .env("OIDC_ISSUER_URL", issuer.url())
            .env("OIDC_CLIENT_ID", "crabbage")
            .env("OIDC_CLIENT_SECRET", "demo-secret")
            .env("CRABBAGE_PUBLIC_URL", format!("http://localhost:{port}"))
            .env("CRABBAGE_DATA_DIR", &data_dir)
            .env("CRABBAGE_LANGUAGE", language)
            .env("LEPTOS_SITE_ADDR", format!("127.0.0.1:{port}"));
        let status = child.status().map_err(|error| error.to_string())?;
        if !keep {
            drop(fs::remove_dir_all(&data_dir));
        }
        if status.success() {
            Ok(())
        } else {
            Err(format!("cargo leptos serve failed: {status}"))
        }
    })
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
