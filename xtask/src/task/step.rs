use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Step {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    pub(crate) env: Vec<(String, String)>,
    pub(crate) expected_stdout: Option<String>,
}

impl Step {
    pub(crate) fn new(program: &str, args: &[&str]) -> Self {
        Self {
            program: program.to_owned(),
            args: args.iter().map(|&arg| arg.to_owned()).collect(),
            env: Vec::new(),
            expected_stdout: None,
        }
    }

    pub(crate) fn cargo(args: &[&str]) -> Self {
        Self::new("cargo", args)
    }

    pub(crate) fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env.push((key.to_owned(), value.to_owned()));
        self
    }

    pub(crate) fn expecting_stdout(mut self, expected: &str) -> Self {
        self.expected_stdout = Some(expected.to_owned());
        self
    }
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (key, value) in &self.env {
            write!(f, "{key}={value} ")?;
        }
        write!(f, "{}", self.program)?;
        for arg in &self.args {
            write!(f, " {arg}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Step;

    #[test]
    fn displays_as_a_shell_line() {
        let step = Step::cargo(&["doc", "--no-deps"]).with_env("RUSTDOCFLAGS", "-D warnings");
        assert_eq!(
            step.to_string(),
            "RUSTDOCFLAGS=-D warnings cargo doc --no-deps"
        );
    }

    #[test]
    fn records_expected_output() {
        let step = Step::new("echo", &["hi"]).expecting_stdout("hi");
        assert_eq!(step.expected_stdout.as_deref(), Some("hi"));
    }
}
