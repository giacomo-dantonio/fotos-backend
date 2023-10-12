use crate::{
    api::error::{ApiResult, ApiError},
    handlers::tags::models,
    AppState
};

use axum::{
    extract::{self, State},
    response::Response, Json, http::StatusCode
};
use std::sync::Arc;

pub async fn tag_path(
    State(state): State<Arc<AppState>>,
    tagid: extract::Path<String>,
    subpath: extract::Path<String>,
) -> ApiResult<Response> {
    Err(ApiError::new(StatusCode::NOT_IMPLEMENTED))
}

pub async fn get_by_tag(
    State(state): State<Arc<AppState>>,
    tag: extract::Path<String>,
    subpath: Option<extract::Path<String>>,
) -> ApiResult<Json<Vec<models::File>>> {
    Err(ApiError::new(StatusCode::NOT_IMPLEMENTED))
}

pub async fn untag_path(
    State(state): State<Arc<AppState>>,
    tag: extract::Path<String>,
    subpath: extract::Path<String>,
) -> ApiResult<Response> {
    Err(ApiError::new(StatusCode::NOT_IMPLEMENTED))
}

#[cfg(test)]
mod tests;
