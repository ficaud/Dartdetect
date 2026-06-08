use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Custom error type that carries an HTTP status code and a human-readable message.
/// Implements `IntoResponse` so it can be returned directly from Axum handlers.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    /// Creates a new `ApiError` with the given status code and message.
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    /// Converts the error into an HTTP response with the stored status code and message body.
    fn into_response(self) -> Response {
        (self.status, self.message).into_response()
    }
}