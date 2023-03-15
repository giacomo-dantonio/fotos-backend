// Credits to angelo.steinbach@stone-dev.de, who is too lazy to publish
// a crate for this.

use axum::{
    http::StatusCode,
    response::{Response, IntoResponse}
};
use tracing::error;

pub type ApiResult<T> = Result<T, ApiError>;

/// A custom error implementation that keeps track of the HTTP error code
/// to be returned by the endpoint.
///
/// This helps keeping the endpoint clean. Even though it scatters
/// the handling of the HTTP request through the code base.
#[derive(Debug)]
pub struct ApiError {
    /// The HTTP error code for the error.
    pub status: StatusCode,

    /// A human readable message to be used as payload for the HTTP response.
    pub message: Option<String>,

    /// The original error.
    pub cause: Option<anyhow::Error>,
}

impl ApiError {
    /// Creates a new `ApiError` for a given code.
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            message: None,
            cause: None,
        }
    }

    /// Adds a message to the `ApiError`.
    pub fn with_msg(mut self, msg: String) -> Self {
        self.message = Some(msg);
        self
    }

    /// Adds a cause to the `ApiError`.
    pub fn with_cause<E>(mut self, cause: E) -> Self
        where
            E: Into<anyhow::Error>,
    {
        self.cause = Some(cause.into());
        self
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match (self.status, self.message) {
            (StatusCode::INTERNAL_SERVER_ERROR, msg) => {
                if let Some(cause) = self.cause {
                    error!("Internal server error: {:?}", cause);
                } else {
                    error!("Internal server error");
                }

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    msg.unwrap_or_else(|| "Something went wrong...".to_string()),
                ).into_response()
            }
            (status, Some(msg)) => (status, msg).into_response(),
            (status, None) => status.into_response(),
        }
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>`
// to turn them into `Result<_, AppError>`.
// That way you don't need to do that manually.
impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>
{
    fn from(err: E) -> Self
    {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
            .with_msg("Something went wrong...".to_string())
            .with_cause(err)
    }
}
