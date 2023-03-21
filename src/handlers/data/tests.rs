use crate::AppState;
use super::Params;

use axum::{
    extract::{Query, State, self},
    http::StatusCode,
    body::{HttpBody}
};
use bytes::Bytes;
use http_body::combinators::UnsyncBoxBody;
use ring::digest::{Context, Digest, SHA256};
use ring::test;
use std::{env, sync::Arc, vec};

#[tokio::test]
async fn get_folder_entries_test() {
    // the get_folder_entries function returns the filenames in the given folder
    let root = env::current_dir()
        .unwrap()
        .join("data");

    let fullpath = super::make_fullpath(root.to_str().unwrap(), None).unwrap();
    let mut actual = super::get_folder_entries(&fullpath)
        .await.unwrap();
    actual.sort();

    let expected = vec![
        "apollon.jpg",
        "folder",
        "penguins.jpg"
    ];

    assert_eq!(actual, expected);
}

fn make_state() -> State<Arc<AppState>> {
    let root = env::current_dir()
        .unwrap()
        .join("data");
    let root = root.to_str().unwrap();

    State(Arc::new(AppState {
        root: root.to_string(),
        connection: "0.0.0.0:3000".to_string(),
        max_level: "DEBUG".to_string()
    }))
}

#[tokio::test]
async fn folder_return_type_test() {
    // if the path is a folder the endpoint will return a json
    let state = make_state();
    let params = Params::default();
    let subpath = extract::Path("folder".to_string());

    let response = super::download(state, Some(subpath), Query(params)).await.unwrap();
    let content_type = response.headers().get("Content-Type").unwrap();

    assert_eq!(content_type.to_str().unwrap(), "application/json");
}

#[tokio::test]
async fn file_return_type_test() {
    // if the path is a file the response headers will contain the content type of the file
    let state = make_state();
    let params = Params::default();
    let subpath = extract::Path("penguins.jpg".to_string());

    let response = super::download(state, Some(subpath), Query(params)).await.unwrap();
    let content_type = response.headers().get("Content-Type").cloned().unwrap();
    assert_eq!(content_type.to_str().unwrap(), "image/jpeg");
}

async fn sha256_digest(body: &mut UnsyncBoxBody<Bytes, axum::Error>) -> anyhow::Result<Digest> {
    let mut context = Context::new(&SHA256);

    while let Some(bytes) = body.data().await {
        let bytes = bytes.unwrap();
        context.update(bytes.as_ref())
    }

    Ok(context.finish())
}

#[tokio::test]
async fn file_return_checksum_test() {
    // if the path is a file the endpoint will return the content of the file
    let state = make_state();
    let params = Params::default();
    let subpath = extract::Path("penguins.jpg".to_string());

    let mut response = super::download(state, Some(subpath), Query(params)).await.unwrap();

    let body = response.body_mut();
    let actual_hash = sha256_digest(body).await.unwrap();

    let expected_hash = "382AD1ABC24D92D8941A38CA3B8B3A2AF9B616D13347F10361C3790D4C78C7E7";
    let expected_hash = test::from_hex(expected_hash).unwrap();

    assert_eq!(&expected_hash, actual_hash.as_ref());
}

#[tokio::test]
async fn not_exists_return_type_test() {
    // if the path doesn't exist the endpoint will return a 404 error code
    let state = make_state();
    let params = Params::default();
    let subpath = extract::Path("not_exists".to_string());

    let result = super::download(state, Some(subpath), Query(params)).await;
    assert!(result.is_err());

    let status = result.unwrap_err();
    assert_eq!(status.status, StatusCode::from_u16(404).unwrap());
}

#[tokio::test]
async fn file_download_name_test() {
    // if the path is a file the browser will download the file with the correct name
    for filename in ["penguins.jpg", "apollon.jpg"] {
        let state = make_state();
        let params = Params::default();

        let subpath = extract::Path(filename.to_string());
        let response = super::download(state, Some(subpath), Query(params)).await.unwrap();
        let content_type = response.headers().get("Content-Disposition").cloned().unwrap();

        assert_eq!(content_type.to_str().unwrap(), format!("attachment; filename=\"{filename}\""));
    }
}

#[tokio::test]
async fn lower_max_dpi_test() {
    // if the path is an image and the max_dpi query parameter is set
    // to a value lower than the image's resolution,
    // the endpoint will lower the resolution of the image.

    unimplemented!()
}

#[tokio::test]
async fn higher_max_dpi_test() {
    // if the path is an image and the max_dpi query parameter is higher than
    // the image's resolution, the endpoint won't lower the resolution of the image.

    unimplemented!()
}

#[tokio::test]
async fn lower_max_width_test() {
    // if the path is an image and the max_width query parameter is set
    // to a value lower than the image's width,
    // the endpoint will resize the image and mantain the ratio.

    unimplemented!()
}

#[tokio::test]
async fn higher_max_width_test() {
    // if the path is an image and the max_width query parameter is higher than
    // the image's width, the endpoint won't resize the image.

    unimplemented!()
}

#[tokio::test]
async fn lower_max_height_test() {
    // if the path is an image and the max_height query parameter is set
    // to a value lower than the image's height,
    // the endpoint will resize the image and mantain the ratio.

    unimplemented!()
}

#[tokio::test]
async fn higher_max_height_test() {
    // if the path is an image and the max_height query parameter is higher than
    // the image's height, the endpoint won't resize the image.

    unimplemented!()
}
