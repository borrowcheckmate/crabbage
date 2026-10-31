use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
/// An error returned while reading from Mealie.
pub enum MealieError {
    /// The base URL is invalid.
    InvalidUrl,
    /// The server refused the API token.
    Unauthorized,
    /// The response is not a Mealie API response.
    NotMealie,
    /// The requested resource does not exist.
    NotFound,
    /// The request timed out.
    Timeout,
    /// The request failed; the message does not contain the API token.
    Network(String),
    /// The server returned another unsuccessful status code.
    Status(u16),
    /// The downloaded image exceeds the size limit.
    TooLarge,
}

impl fmt::Display for MealieError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUrl => {
                f.write_str("the Mealie URL must be an HTTP or HTTPS URL with a host")
            }
            Self::Unauthorized => f.write_str("the API token was refused"),
            Self::NotMealie => f.write_str("the server is not a Mealie server"),
            Self::NotFound => f.write_str("the requested Mealie resource was not found"),
            Self::Timeout => f.write_str("the Mealie server did not answer in time"),
            Self::Network(message) => {
                write!(f, "the Mealie server could not be reached: {message}")
            }
            Self::Status(code) => write!(f, "the Mealie server answered with status {code}"),
            Self::TooLarge => f.write_str("the Mealie image is too large"),
        }
    }
}

impl std::error::Error for MealieError {}
