use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use crabbage_store::StoreError;

pub(crate) enum AuthError {
    Rejected(&'static str),
    Internal,
    Unavailable,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            Self::Rejected(message) => (StatusCode::BAD_REQUEST, message).into_response(),
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Self::Unavailable => StatusCode::SERVICE_UNAVAILABLE.into_response(),
        }
    }
}

impl From<Box<dyn std::error::Error>> for AuthError {
    fn from(_: Box<dyn std::error::Error>) -> Self {
        Self::Unavailable
    }
}

impl From<StoreError> for AuthError {
    fn from(_: StoreError) -> Self {
        Self::Internal
    }
}

impl From<getrandom::Error> for AuthError {
    fn from(_: getrandom::Error) -> Self {
        Self::Internal
    }
}
