use std::fmt;

use super::Name;

/// A greeting addressed to a [`Name`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Greeting {
    name: Name,
}

impl Greeting {
    /// Creates a greeting for `name`.
    #[must_use]
    pub const fn new(name: Name) -> Self {
        Self { name }
    }

    /// The person being greeted.
    #[must_use]
    pub const fn name(&self) -> &Name {
        &self.name
    }
}

impl fmt::Display for Greeting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hello, {}!", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::{Greeting, Name};

    #[test]
    fn greets_by_name() {
        let name: Name = "Ada".parse().unwrap();
        assert_eq!(Greeting::new(name).to_string(), "Hello, Ada!");
    }

    #[test]
    fn keeps_the_name() {
        let name: Name = "Ada".parse().unwrap();
        assert_eq!(Greeting::new(name.clone()).name(), &name);
    }
}
