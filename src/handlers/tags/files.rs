use crate::{
    api::error::ApiResult,
    AppState
};

use axum::{
    extract::{self, State},
    response::Response
};
use std::sync::Arc;
use uuid::Uuid;

pub async fn tag_path(
    State(state): State<Arc<AppState>>,
    tagid: extract::Path<String>,
    subpath: extract::Path<String>,
) -> ApiResult<Response> {
    unimplemented!()
}

pub async fn get_by_tag(
    State(state): State<Arc<AppState>>,
    tag: extract::Path<String>,
    subpath: Option<extract::Path<String>>,
) -> ApiResult<Response> {
    unimplemented!()
}

pub async fn untag_path(
    State(state): State<Arc<AppState>>,
    tag: extract::Path<String>,
    subpath: extract::Path<String>,
) -> ApiResult<Response> {
    unimplemented!()
}

#[cfg(test)]
mod tests;
