// FIXME: do something like this, but using a macro: https://medium.com/@ericdreichert/test-setup-and-teardown-in-rust-without-a-framework-ba32d97aa5ab

extern crate test_macro;

use crate::test_utils::{make_state, insert_tags};
use axum::{extract, http::StatusCode};
use rstest::rstest;
use sqlx::{Row, SqlitePool};
use test_macro::run_test;
use uuid::Uuid;

async fn setup() {}

async fn teardown() {}

async fn get_tag_id(pool: &SqlitePool, tagname: &str) -> String {
    let x = (setup, teardown);

    sqlx::query("SELECT id FROM tags WHERE tagname=$1")
        .bind(tagname)
        .fetch_one(pool).await
        .expect("Cannot fetch tag id")
        .get("id")
}

#[tokio::test]
#[run_test]
async fn test_tag_path_files_record() {
    // the tag_path endpoint creates a record in the files table, if that doesn't exist yet

    let state = make_state().await;
    let pool = state.pool.clone();

    // Seed the database
    let tagnames = ["Landscape", "Sea", "Mountain"];
    insert_tags(tagnames.into_iter(), &pool).await;

    // Get endpoint parameters
    let tag_id = get_tag_id(&pool, &tagnames[0]).await;
    let subpath = "penguins.jpg";

    // Call the endpoint
    let response =
        super::tag_path(state, extract::Path(tag_id), extract::Path(subpath.to_string())).await
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
#[case(true, false)]
#[case(true, true)]
#[tokio::test]
#[run_test]
async fn test_tag_path_filetag(#[case] files_exists: bool, #[case] filetags_exists: bool) {
    // the tag_path endpoint creates a record in the filetags table
    // If `files_exists` an entry in the files table for the path is created before calling the endpoint
    // If `filetags_exists` an entry in the filetags table for the path is created before calling the endpoint
    let state = make_state().await;
    let pool = state.pool.clone();

    let tagnames = ["Test Tag"];
    insert_tags(tagnames.into_iter(), &pool).await;
    let tag_id = get_tag_id(&pool, &tagnames[0]).await;

    let subpath = "penguins.jpg";
    if files_exists || filetags_exists {
        let file_id = Uuid::new_v4();
        let checksum = "382AD1ABC24D92D8941A38CA3B8B3A2AF9B616D13347F10361C3790D4C78C7E7";
        sqlx::query("INSERT INTO files (id, relative_path, csum) VALUES ($1, $2, $3)")
            .bind(file_id.to_string()).bind(subpath).bind(checksum)
            .execute(&pool).await
            .expect("Cannot insert file");

        if filetags_exists {
            sqlx::query("INSERT INTO filetags (tag_id, file_id) VALUES ($1, $2)")
                .bind(&tag_id).bind(file_id.to_string())
                .execute(&pool).await
                .expect("Cannot insert tag");
        }
    }

    // Call the endpoint
    let response =
        super::tag_path(state, extract::Path(tag_id), extract::Path(subpath.to_string())).await
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

#[tokio::test]
#[run_test]
async fn test_tag_path_file_doesnt_exist() {
    // the tag_path endpoint responds with a 404, if the path doesn't exist
    let state = make_state().await;
    let pool = state.pool.clone();

    let tagnames = ["Test Tag"];
    insert_tags(tagnames.into_iter(), &pool).await;
    
    let tag_id = get_tag_id(&pool, &tagnames[0]).await;
    let subpath = "molise.jpg";

    // Call the endpoint
    let response =
        super::tag_path(state, extract::Path(tag_id), extract::Path(subpath.to_string())).await;
    assert!(response.is_err_and(|apierr| apierr.status == StatusCode::NOT_FOUND));
}

#[tokio::test]
#[run_test]
async fn test_tag_path_tag_doesnt_exist() {
    // the tag_path endpoint responds with a 404, if the tag doesn't exist
    let state = make_state().await;

    let tag_id = Uuid::new_v4().to_string();
    let subpath = "penguins.jpg";

    // Call the endpoint
    let response =
        super::tag_path(state, extract::Path(tag_id), extract::Path(subpath.to_string())).await;
    assert!(response.is_err_and(|apierr| apierr.status == StatusCode::NOT_FOUND));
}

#[tokio::test]
#[run_test]
async fn test_untag_path() {
    // the untag_path endpoint deletes the filetags record, if the path exists and is tagged
    let state = make_state().await;
    let pool = state.pool.clone();

    let tagnames = ["Test Tag"];
    insert_tags(tagnames.into_iter(), &pool).await;
    let tag_id = get_tag_id(&pool, &tagnames[0]).await;

    let subpath = "penguins.jpg";
    let file_id = Uuid::new_v4();
    let checksum = "382AD1ABC24D92D8941A38CA3B8B3A2AF9B616D13347F10361C3790D4C78C7E7";
    sqlx::query("INSERT INTO files (id, relative_path, csum) VALUES ($1, $2, $3)")
        .bind(file_id.to_string()).bind(subpath).bind(checksum)
        .execute(&pool).await
        .expect("Cannot insert file");

    sqlx::query("INSERT INTO filetags (tag_id, file_id) VALUES ($1, $2)")
        .bind(&tag_id).bind(file_id.to_string())
        .execute(&pool).await
        .expect("Cannot insert tag");

    // Call the endpoint
    let response =
        super::untag_path(state, extract::Path(tag_id.clone()), extract::Path(subpath.to_string())).await
        .expect("Endpoint returned an Err");
    assert_eq!(StatusCode::OK, response.status());

    // Check assertion on the database
    let row = sqlx::query(
            "SELECT * FROM filetags WHERE tag_id=$1 and file_id=$2"
        ).bind(&tag_id).bind(file_id.to_string())
        .fetch_optional(&pool)
        .await
        .expect("Error while querying the database");
    assert!(row.is_none());
}

#[tokio::test]
#[run_test]
async fn test_untag_path_path_not_found() {
    // the untag_path endpoint responds with a 404, if the path doesn't exist
    let state = make_state().await;
    let pool = state.pool.clone();

    let tagnames = ["Test Tag"];
    insert_tags(tagnames.into_iter(), &pool).await;
    
    let tag_id = get_tag_id(&pool, &tagnames[0]).await;
    let subpath = "molise.jpg";

    // Call the endpoint
    let response =
        super::untag_path(state, extract::Path(tag_id), extract::Path(subpath.to_string())).await;
    assert!(response.is_err_and(|apierr| apierr.status == StatusCode::NOT_FOUND));
}

#[tokio::test]
#[run_test]
async fn test_untag_path_tag_not_found() {
    // the untag_path endpoint responds with a 404, if the tag doesn't exist
    let state = make_state().await;

    let tag_id = Uuid::new_v4().to_string();
    let subpath = "penguins.jpg";

    // Call the endpoint
    let response =
        super::untag_path(state, extract::Path(tag_id), extract::Path(subpath.to_string())).await;
    assert!(response.is_err_and(|apierr| apierr.status == StatusCode::NOT_FOUND));

}

#[tokio::test]
#[run_test]
async fn test_untag_path_path_not_tagged() {
    // the untag_path endpoint responds with a 404, if the path exists but it's not tagged with that tag
    let state = make_state().await;
    let pool = state.pool.clone();

    let tagnames = ["Test Tag"];
    insert_tags(tagnames.into_iter(), &pool).await;
    
    let tag_id = get_tag_id(&pool, &tagnames[0]).await;
    let subpath = "penguins.jpg";

    // Call the endpoint
    let response =
        super::untag_path(state, extract::Path(tag_id), extract::Path(subpath.to_string())).await;
    assert!(response.is_err_and(|apierr| apierr.status == StatusCode::NOT_FOUND));
}

#[tokio::test]
#[run_test]
async fn test_get_by_tag_no_path() {
    // the get_by_tag endpoint returns all the filepaths which are tagged with that tag,
    // if the tag exists and no subpath is provided.
    let state = make_state().await;
    let pool = state.pool.clone();

    let tagnames = ["Test Tag"];
    insert_tags(tagnames.into_iter(), &pool).await;
    let tag_id = get_tag_id(&pool, &tagnames[0]).await;

    let features = [
        ("penguins.jpg", Uuid::new_v4(), "382AD1ABC24D92D8941A38CA3B8B3A2AF9B616D13347F10361C3790D4C78C7E7"),
        ("apollon.jpg", Uuid::new_v4(), "0911A4647AC2CE5CAF95DF11A56920D2CAE2A982C259B7608F73081167BC1867")
    ];
    for (subpath, file_id, checksum) in features {
        sqlx::query("INSERT INTO files (id, relative_path, csum) VALUES ($1, $2, $3)")
            .bind(file_id.to_string()).bind(subpath).bind(checksum)
            .execute(&pool).await
            .expect("Cannot insert file");

        sqlx::query("INSERT INTO filetags (tag_id, file_id) VALUES ($1, $2)")
            .bind(&tag_id).bind(file_id.to_string())
            .execute(&pool).await
            .expect("Cannot insert tag");
    }

    let response = super::get_by_tag(state, extract::Path(tag_id), None)
        .await
        .expect("Failed to get files by tag.");

    let actual: Vec<&str> = response.iter().map(|f| f.relative_path.as_str()).collect();
    assert_eq!(vec!["apollon.jpg", "penguins.jpg"], actual);
}

#[tokio::test]
#[run_test]
async fn test_get_by_tag_with_path() {
    // the get_by_tag endpoint returns all the filepaths under the subpath which are tagged with that tag, if the tag exists and an existing subpath is provided
    unimplemented!();
}

#[tokio::test]
#[run_test]
async fn test_get_by_tag_tag_doesnt_exist() {
    // the get_by_tag endpoint with a 404, if the tag doesn't exists
    unimplemented!();
}

#[tokio::test]
#[run_test]
async fn test_get_by_tag_path_doesnt_exist() {
    // the get_by_tag endpoint with a 404, if a non existing subpath is provided
    unimplemented!();
}
