use std::error::Error;
use std::fmt;
use std::str::FromStr;

/// A trimmed, non-empty name of at most [`Name::MAX_CHARS`] characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name(String);

/// Why an input was rejected as a [`Name`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameError {
    /// The input was empty or contained only whitespace.
    Empty,
    /// The input was longer than [`Name::MAX_CHARS`] characters.
    TooLong,
}

impl Name {
    /// Longest accepted name, counted in characters.
    pub const MAX_CHARS: usize = 64;

    /// The name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Name {
    type Err = NameError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(NameError::Empty);
        }
        if trimmed.chars().count() > Self::MAX_CHARS {
            return Err(NameError::TooLong);
        }
        Ok(Self(trimmed.to_owned()))
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for NameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("name must not be empty"),
            Self::TooLong => write!(f, "name must be at most {} characters", Name::MAX_CHARS),
        }
    }
}

impl Error for NameError {}

#[cfg(test)]
mod tests {
    use super::{Name, NameError};

    #[test]
    fn trims_surrounding_whitespace() {
        let name: Name = "  Ada \n".parse().unwrap();
        assert_eq!(name.as_str(), "Ada");
    }

    #[test]
    fn rejects_blank_input() {
        assert_eq!(" \t ".parse::<Name>(), Err(NameError::Empty));
    }

    #[test]
    fn accepts_exactly_max_chars() {
        let input = "é".repeat(Name::MAX_CHARS);
        assert_eq!(input.parse::<Name>().unwrap().as_str(), input);
    }

    #[test]
    fn rejects_more_than_max_chars() {
        let input = "a".repeat(Name::MAX_CHARS + 1);
        assert_eq!(input.parse::<Name>(), Err(NameError::TooLong));
    }

    #[test]
    fn displays_as_plain_text() {
        let name: Name = "Grace".parse().unwrap();
        assert_eq!(name.to_string(), "Grace");
    }

    #[test]
    fn errors_explain_the_limit() {
        assert_eq!(NameError::Empty.to_string(), "name must not be empty");
        assert_eq!(
            NameError::TooLong.to_string(),
            "name must be at most 64 characters"
        );
    }
}
