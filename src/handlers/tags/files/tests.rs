use crate::test_utils::{setup, make_state, insert_tags};
use axum::{extract, http::StatusCode};
use rstest::rstest;
use sqlx::Row;

#[tokio::test]
async fn test_tag_path_files_record() {
    // the tag_path endpoint creates a record in the files table, if that doesn't exist yet

    let state = make_state().await;
    let pool = state.pool.clone();

    setup(&pool).await;

    // Seed the database
    let tagnames = [
        "Landscape".to_string(),
        "Sea".to_string(),
        "Mountain".to_string()
    ];
    insert_tags(
        tagnames.iter().map(|s| s.as_str()),
        &state.pool
    ).await;

    // Get endpoint parameters
    let tagid : String = sqlx::query("SELECT id FROM tags WHERE tagname=$1")
        .bind(&tagnames[0])
        .fetch_one(&pool).await
        .expect("Cannot fetch tag id")
        .get("id");
    let subpath = "penguins.jpg".to_string();

    // Call the endpoint
    let response =
        super::tag_path(state, extract::Path(tagid), extract::Path(subpath.clone())).await
        .expect("Endpoint returned an Err");
    assert_eq!(StatusCode::OK, response.status());

    // Check assertion on the database
    let row = sqlx::query(
            "SELECT * FROM files WHERE relative_path=$1 LIMIT 1"
        ).bind(subpath)
        .fetch_optional(&pool)
        .await
        .expect("Error while querying the database");
    assert!(row.is_some());
}

#[rstest]
#[case(false, false)]
#[case(false, true)]
#[case(true, false)]
#[case(true, true)]
#[tokio::test]
async fn test_tag_path_filetag(#[case] files_exists: bool, #[case] filetags_exists: bool) {
    // the tag_path endpoint creates a record in the filetags table
    // If `files_exists` an entry in the files table for the path is created before calling the endpoint
    // If `filetags_exists` an entry in the filetags table for the path is created before calling the endpoint
    unimplemented!();
}

#[tokio::test]
async fn test_tag_path_file_doesnt_exist() {
    // the tag_path endpoint responds with a 404, if the path doesn't exist
    unimplemented!();
}

#[tokio::test]
async fn test_tag_path_tag_doesnt_exist() {
    // the tag_path endpoint responds with a 404, if the tag doesn't exist
    unimplemented!();
}

#[tokio::test]
async fn test_untag_path() {
    // the untag_path endpoint deletes the filetags record, if the path exists and is tagged
    unimplemented!();
}

#[tokio::test]
async fn test_untag_path_path_not_found() {
    // the untag_path endpoint responds with a 404, if the path doesn't exist
    unimplemented!();
}

#[tokio::test]
async fn test_untag_path_tag_not_found() {
    // the untag_path endpoint responds with a 404, if the tag doesn't exist
    unimplemented!();
}

#[tokio::test]
async fn test_untag_path_path_not_tagged() {
    // the untag_path endpoint responds with a ?, if the path exists but it's not tagged with that tag
    unimplemented!();
}

#[tokio::test]
async fn test_get_by_tag_no_path() {
    // the get_by_tag endpoint returns all the filepaths which are tagged with that tag, if the tag exists and no subpath is provided
    unimplemented!();
}

#[tokio::test]
async fn test_get_by_tag_with_path() {
    // the get_by_tag endpoint returns all the filepaths under the subpath which are tagged with that tag, if the tag exists and an existing subpath is provided
    unimplemented!();
}

#[tokio::test]
async fn test_get_by_tag_tag_doesnt_exist() {
    // the get_by_tag endpoint with a 404, if the tag doesn't exists
    unimplemented!();
}

#[tokio::test]
async fn test_get_by_tag_path_doesnt_exist() {
    // the get_by_tag endpoint with a 404, if a non existing subpath is provided
    unimplemented!();
}
