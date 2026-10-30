use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

/// Every problem found while reading the configuration.
pub struct ConfigError {
    problems: Vec<Problem>,
}

impl ConfigError {
    pub(super) const fn new(problems: Vec<Problem>) -> Self {
        Self { problems }
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("invalid configuration")?;
        for problem in &self.problems {
            write!(f, "\n  - {problem}")?;
        }
        Ok(())
    }
}

impl Debug for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl Error for ConfigError {}

pub(super) enum Problem {
    Missing(&'static str),
    Invalid { name: &'static str, reason: String },
}

impl Display for Problem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(name) => write!(f, "{name} is not set"),
            Self::Invalid { name, reason } => write!(f, "{name} is invalid: {reason}"),
        }
    }
}
