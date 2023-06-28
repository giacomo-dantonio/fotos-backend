use axum::{extract::{Query, Path}, Json};
use rstest::rstest;
use sqlx::sqlite::SqlitePool;
use uuid::Uuid;

use crate::{
    handlers::tags::models::Tag,
    test_utils::{setup, make_state}
};

async fn insert_tags(names: impl Iterator<Item=&str>, pool: &SqlitePool) {
    // FIXME: this has concurrency issues when the tests run in parallel
    sqlx::query("DELETE FROM tags WHERE 1=1")
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

    let tagnames = [
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

    assert_eq!(tagnames, *actual);
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

#[rstest]
#[case(false)]
#[case(true)]
#[tokio::test]
async fn test_create_tag(#[case] duplicate: bool) {
    // The endpoint will create a new tag in the database,
    // if not already there.

    let state = make_state().await;
    setup(&state.pool).await;

    let tagname = "next_tag";
    let existing_tags = if duplicate {
        vec![tagname]
    } else {
        vec![]
    };
    insert_tags(existing_tags.into_iter(), &state.pool).await;

    let params = super::Params::default();
    let response: Json<Vec<Tag>> = super::get_tags(state.clone(), Query(params))
        .await
        .expect("Failed to get tags from the handler");
    assert_eq!((*response).len(), if duplicate { 1 } else { 0 });

    let response = super::create_tag(state.clone(), Path(tagname.to_string()))
        .await;

    if !duplicate {
        let response = response.expect("Failed to create the tag");        
        assert_eq!((*response).tagname, tagname);
    } else if let Err(error) = response {
        assert_eq!(error.status, 409);
    } else {
        assert!(false, "Response should have an error status code");
    }
}