//! How the admin pages show a failed action or a failed load.

use leptos::prelude::*;

/// The notice of the last run of an action, when that run failed.
pub(crate) fn failure<T>(value: Option<&Result<T, ServerFnError>>) -> Option<AnyView> {
    match value {
        Some(Err(error)) => Some(error_notice(&message(error))),
        Some(Ok(_)) | None => None,
    }
}

/// A failure notice with the shared error style.
pub(crate) fn error_notice(text: &str) -> AnyView {
    view! { <p class="notice error">{text.to_owned()}</p> }.into_any()
}

/// The text the server sent with a failure.
pub(crate) fn message(error: &ServerFnError) -> String {
    if let ServerFnError::ServerError(text) = error {
        return text.clone();
    }
    error.to_string()
}
