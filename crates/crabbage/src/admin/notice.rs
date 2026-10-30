//! How the admin pages show a failed action or a failed load.

use leptos::prelude::ServerFnError;

/// The message of the last run of an action, when that run failed.
pub(crate) fn failure<T>(value: Option<&Result<T, ServerFnError>>) -> Option<String> {
    match value {
        Some(Err(error)) => Some(message(error)),
        Some(Ok(_)) | None => None,
    }
}

/// The text the server sent with a failure.
pub(crate) fn message(error: &ServerFnError) -> String {
    if let ServerFnError::ServerError(text) = error {
        return text.clone();
    }
    error.to_string()
}
