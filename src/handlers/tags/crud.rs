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
) -> ApiResult<Response> {
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
mod tests {
    use axum::{extract::Query, Json};
    use sqlx::sqlite::SqlitePool;
    use uuid::Uuid;

    use crate::{
        handlers::tags::models::Tag,
        test_utils::{setup, make_state}
    };

    async fn insert_tags(names: impl Iterator<Item=&str>, pool: &SqlitePool) {
        sqlx::query("DELETE FROM tags")
            .execute(pool)
            .await
            .expect("Unable to delete old tags");
    
        for name in names {
            let id = Uuid::new_v4();
            sqlx::query("INSERT INTO tags (id, tagname) VALUES ($1, $2)")
                .bind(id.to_string())
                .bind(name)
                .execute(pool)
                .await
                .expect("Unable to insert tag");
        }
    }

    #[tokio::test]
    async fn test_get_tags() {
        // the endpoint without query parameters will return all tags
        let state = make_state().await;
        setup(&state.pool).await;

        let tagnames = vec![
            "Landscape".to_string(),
            "Sea".to_string(),
            "Mountain".to_string()
        ];
        insert_tags(
            tagnames.iter().map(|s| s.as_str()),
            &state.pool
        ).await;
    
        let params = super::Params::default();
        let response: Json<Vec<Tag>> = super::get_tags(state, Query(params))
            .await
            .expect("Failed to get tags from the handler");

        let actual = (*response).clone();
        let actual: Vec<String> = actual
            .into_iter()
            .map(|t| t.tagname)
            .collect();

        assert_eq!(tagnames, actual);
    }

    #[tokio::test]
    async fn test_query_tags() {
        // the endpoint with a query parameter will filter the tags
        // according to the search string
        let state = make_state().await;
        setup(&state.pool).await;

        let tagnames = vec![
            "Landscape".to_string(),
            "Sea".to_string(),
            "Mountain".to_string()
        ];
        insert_tags(
            tagnames.iter().map(|s| s.as_str()),
            &state.pool
        ).await;
    
        let mut params = super::Params::default();
        params.query = Some("OUNT".to_string());

        let response: Json<Vec<Tag>> = super::get_tags(state, Query(params))
            .await
            .expect("Failed to get tags from the handler");

        let expected = vec!["Mountain".to_string()];
        let actual = (*response).clone();
        let actual: Vec<String> = actual
            .into_iter()
            .map(|t| t.tagname)
            .collect();

        assert_eq!(expected, actual);
    }

    // TODO: tests for create tags
}