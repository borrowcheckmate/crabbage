use std::fmt;

/// Why a page could not be imported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportError {
    /// The address is not an http or https URL.
    NotHttp,
    /// The address resolves to a private, loopback or otherwise non-public host.
    PrivateAddress,
    /// The page did not answer within the time limit.
    Timeout,
    /// The page is larger than the size limit.
    TooLarge,
    /// The page answered with a non-success status code.
    Status(u16),
    /// The connection failed; the text says why.
    Network(String),
    /// The page has no schema.org `Recipe` data.
    NoRecipe,
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotHttp => f.write_str("only http and https addresses can be imported"),
            Self::PrivateAddress => f.write_str("the address is not a public internet address"),
            Self::Timeout => f.write_str("the page took too long to answer"),
            Self::TooLarge => f.write_str("the page is larger than the import limit"),
            Self::Status(code) => write!(f, "the page answered with status {code}"),
            Self::Network(message) => write!(f, "the page could not be fetched: {message}"),
            Self::NoRecipe => f.write_str("the page has no schema.org Recipe data"),
        }
    }
}

impl std::error::Error for ImportError {}
