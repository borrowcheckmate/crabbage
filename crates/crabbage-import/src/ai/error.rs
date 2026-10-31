use std::fmt;

/// Why the AI service gave no usable draft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiError {
    /// The service did not answer within the time limit.
    Timeout,
    /// The service answered with a non-success status code.
    Status(u16),
    /// The connection failed; the text says why.
    Network(String),
    /// The answer is not a usable draft; the text says why.
    Invalid(String),
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout => f.write_str("the AI service did not answer in time"),
            Self::Status(401 | 403) => f.write_str("the AI service rejected the API key"),
            Self::Status(429) => f.write_str("the AI service is busy, try again later"),
            Self::Status(code) => write!(f, "the AI service answered with status {code}"),
            Self::Network(message) => write!(f, "the AI service could not be reached: {message}"),
            Self::Invalid(reason) => write!(f, "the AI service gave an unusable draft: {reason}"),
        }
    }
}

impl std::error::Error for AiError {}
