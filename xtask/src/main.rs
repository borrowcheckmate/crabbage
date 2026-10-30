//! Repository automation. Run `cargo xtask --help` for the available tasks.

mod cli;
mod policy;
mod rename;
mod repo;
mod task;

use std::io::{self, Write};
use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let cli = cli::Cli::parse();
    match task::run(&cli.command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            if writeln!(io::stderr(), "error: {message}").is_err() {
                return ExitCode::FAILURE;
            }
            ExitCode::FAILURE
        }
    }
}
