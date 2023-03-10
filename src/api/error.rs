// Credits to angelo.steinbach@stone-dev.de, who is to lazy to publish
// a crate for this.

use axum::{
    http::StatusCode,
    IntoResponse,
    Response
};
use tracing::error;

pub type ApiResult<T> = Result<T, ApiError>;

pub struct ApiError {
    status: StatusCode,
    message: Option<&'static str>,
    cause: Option<anyhow::Error>,
}

impl ApiError {
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            message: None,
            cause: None,
        }
    }

    pub fn with_msg(mut self, msg: &'static str) -> Self {
        self.message = Some(msg);
        self
    }

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
                    msg.unwrap_or("Something went wrong..."),
                ).into_response()
            }
            (status, Some(msg)) => (status, msg).into_response(),
            (status, None) => status.into_response(),
        }
    }
}
// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into// `Result<_, AppError>`. That way you don't need to do that manually.impl<E> From<E> for ApiErrorwhere    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
            .with_msg("Something went wrong...")
            .with_cause(err)
    }
}
