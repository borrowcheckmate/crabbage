//! Prints a greeting for the name given as arguments.

use std::env;
use std::fmt::Display;
use std::io::{self, Write};
use std::process::ExitCode;

use crabbage::{Greeting, Name};

const DEFAULT_NAME: &str = "world";
const USAGE_ERROR: u8 = 2;

fn main() -> ExitCode {
    let words: Vec<String> = env::args_os()
        .skip(1)
        .map(|word| word.to_string_lossy().into_owned())
        .collect();
    let input = if words.is_empty() {
        DEFAULT_NAME.to_owned()
    } else {
        words.join(" ")
    };
    match input.parse::<Name>() {
        Ok(name) => report(&mut io::stdout(), &Greeting::new(name), ExitCode::SUCCESS),
        Err(error) => report(&mut io::stderr(), &error, ExitCode::from(USAGE_ERROR)),
    }
}

fn report(out: &mut impl Write, message: &impl Display, code: ExitCode) -> ExitCode {
    if writeln!(out, "{message}").is_err() {
        return ExitCode::FAILURE;
    }
    code
}
