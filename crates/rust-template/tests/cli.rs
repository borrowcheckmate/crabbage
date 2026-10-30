//! Runs the compiled binary the way a user would.

use std::borrow::Cow;
use std::io;
use std::process::{Command, Output};

fn run(args: &[&str]) -> io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_rust-template"))
        .args(args)
        .output()
}

fn stdout(output: &Output) -> Cow<'_, str> {
    String::from_utf8_lossy(&output.stdout)
}

fn stderr(output: &Output) -> Cow<'_, str> {
    String::from_utf8_lossy(&output.stderr)
}

#[test]
fn greets_the_world_without_arguments() {
    let output = run(&[]).unwrap();
    assert!(output.status.success());
    assert_eq!(stdout(&output), "Hello, world!\n");
}

#[test]
fn joins_arguments_into_one_name() {
    let output = run(&["Ada", "Lovelace"]).unwrap();
    assert!(output.status.success());
    assert_eq!(stdout(&output), "Hello, Ada Lovelace!\n");
}

#[test]
fn rejects_a_blank_name_with_usage_error() {
    let output = run(&["   "]).unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "name must not be empty\n");
}

#[test]
fn rejects_a_name_that_is_too_long() {
    let output = run(&[&"a".repeat(65)]).unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stderr(&output), "name must be at most 64 characters\n");
}
