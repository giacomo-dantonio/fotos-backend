use crate::{
    api::error::ApiResult,
    AppState,
    handlers::tags::models::Tag
};

use axum::{
    extract::{Query, State, Path},
    response::Response, Json
};
use serde::Deserialize;
use std::sync::Arc;

/// Search for tags.
/// 
/// Handles request of the type
/// 
/// #    GET /tags
/// #    GET /tags?search=pony
///
/// The endpoint returns the list of available tags.
/// The list is filtered by the search term, if provided.
///
/// # Arguments
///
/// - `State(state)` - The shared state of the application.
/// - `params` - Specify the optional search term.
///
/// # Returns
/// A list of the search results
pub async fn get_tags(
    State(state): State<Arc<AppState>>,
    Query(params): Query<Params>
) -> ApiResult<Json<Vec<Tag>>> {
    let query = if let Some(search_string) = params.query {
        format!("SELECT * FROM tags WHERE LOWER(tagname) LIKE '%{}%'", search_string.to_lowercase())
    } else {
        "SELECT * FROM tags".to_string()
    };

    let tags = sqlx::query_as(&query)
        .fetch_all(&state.pool)
        .await?;

    Ok(tags.into())
}

pub async fn create_tag(
    State(state): State<Arc<AppState>>,
    Path(tagname): Path<String>
) -> ApiResult<Json<Tag>> {
    let count: u64 = sqlx::query_as(
        "SELECT COUNT(*) FROM tags WHERE tagname=$1"
    ).bind(tagname)
    .fetch_one(&state.pool)
    .await?;

    unimplemented!()
}

/// Query parameters for the get_tags endpoint.
/// 
/// - `search` - If provided only the tags that match the search string
///     will be returned.
#[derive(Default, Deserialize)]
pub struct Params {
    query: Option<String>,
}

#[cfg(test)]
mod tests;