use crate::AppState;

use anyhow::{anyhow, Result};
use axum::{
    body::StreamBody,
    extract::{self, State},
    http::{StatusCode, header},
    Json,
    response::{IntoResponse, Response}
};
use tokio_stream::StreamExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio_stream::wrappers::ReadDirStream;
use tokio_util::io::ReaderStream;

// FIXME: add documentation
// FIXME: add logging

fn make_fullpath(root: &str, subpath: Option<&str>) -> Result<PathBuf>
{
    let mut fullpath = Path::new(root).to_path_buf();
    if let Some(subpath) = subpath {
        fullpath = fullpath.join(subpath);
    }

    if fullpath.exists() {
        Ok(fullpath)
    }
    else {
        Err(anyhow!("path {} doesn't exist", fullpath.to_str().unwrap_or("")))
    }
}

async fn is_dir(path: &PathBuf) -> Result<bool>
{
    let metadata = fs::metadata(path).await?;
    Ok(metadata.is_dir())
}

async fn get_folder_entries(fullpath: &PathBuf) -> Result<Vec<String>> {
    let entries = fs::read_dir(fullpath).await?;
    let mut entries = ReadDirStream::new(entries);

    let mut result = vec![];
    while let Some(entry) = entries.next().await {
        if let Ok(entry) = entry {
            let filename = entry.file_name().to_str()
                .ok_or(anyhow!("Encoding issue"))?
                .to_string();
            result.push(filename);
        }
    }

    Ok(result)
}

async fn get_file_stream(fullpath: &PathBuf) -> Result<impl IntoResponse> {
    // Based on https://github.com/tokio-rs/axum/discussions/608

    // `File` implements `AsyncRead`
    let file = tokio::fs::File::open(fullpath).await?;

    let filename = fullpath.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let header_filename = format!("attachment; filename=\"{filename}\"");

    let headers = [
        (header::CONTENT_TYPE, "application/octet-stream".to_string()),
        (
            header::CONTENT_DISPOSITION,
            header_filename,
        ),
    ];

    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = StreamBody::new(stream);

    Ok((headers, body))
}

pub async fn folder_list(
    State(cfg): State<Arc<AppState>>,
    subpath: Option<extract::Path<String>>) -> Result<Response, StatusCode> {

    let subpath = subpath.as_ref().map(|p| p.as_str());
    let fullpath = make_fullpath(&cfg.root, subpath)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let is_dir = is_dir(&fullpath).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let result: Result<Response>;
    if is_dir {
        result = get_folder_entries(&fullpath).await
            .map(|children| Json(children).into_response());
    }
    else {
        result = get_file_stream(&fullpath).await
            .map(|stream| stream.into_response());
    }

    result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use http_body::combinators::UnsyncBoxBody;
    use std::{env, sync::Arc, vec};
    use axum::{extract::{State, self}, http::StatusCode, body::{HttpBody}};
    use crate::AppState;
    use ring::digest::{Context, Digest, SHA256};
    use ring::test;

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

        State(Arc::new(AppState { root: root.to_string() }))
    }

    #[tokio::test]
    async fn folder_return_type_test() {
        // if the path is a folder the endpoint will return a json
        let state = make_state();
        let subpath = extract::Path("folder".to_string());

        let response = super::folder_list(state, Some(subpath)).await.unwrap();
        let content_type = response.headers().get("Content-Type").unwrap();

        assert_eq!(content_type.to_str().unwrap(), "application/json");
    }

    #[tokio::test]
    async fn file_return_type_test() {
        // if the path is a file the endpoint will return the content of the file
        let state = make_state();
        let subpath = extract::Path("penguins.jpg".to_string());

        let response = super::folder_list(state, Some(subpath)).await.unwrap();
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
        let subpath = extract::Path("penguins.jpg".to_string());

        let mut response = super::folder_list(state, Some(subpath)).await.unwrap();

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
        let subpath = extract::Path("not_exists".to_string());

        let result = super::folder_list(state, Some(subpath)).await;
        assert!(result.is_err());

        let status = result.unwrap_err();
        assert_eq!(status, StatusCode::from_u16(404).unwrap());
    }

    #[tokio::test]
    async fn file_download_name_test() {
        // if the path is a file the browser will download the file with the correct name
        for filename in ["penguins.jpg", "apollon.jpg"] {
            let state = make_state();

            let subpath = extract::Path(filename.to_string());
            let response = super::folder_list(state, Some(subpath)).await.unwrap();
            let content_type = response.headers().get("Content-Disposition").cloned().unwrap();

            assert_eq!(content_type.to_str().unwrap(), format!("attachment; filename=\"{filename}\""));
        }
    }
}
