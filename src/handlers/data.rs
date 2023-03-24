use crate::{
    api::error::{ApiError, ApiResult},
    AppState
};

use axum::{
    body::StreamBody,
    extract::{self, Query, State},
    http::{StatusCode, header},
    Json,
    response::{IntoResponse, Response}
};
use mime::Mime;
use mime_guess;
use serde::Deserialize;
use tokio_stream::StreamExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio_stream::wrappers::ReadDirStream;
use tokio_util::io::ReaderStream;

/// Handles the route for the path specified by `subpath` by returning the
/// content of the ressource.
/// 
/// For example if the root folder is `/opt/content`
/// and the client does an http request like
/// 
/// #    GET /my/little/pony
///
/// The endpoint will return the content of the file
/// `/opt/content/my/little/pony`.
/// If it is a folder, it will return a json response containing the list
/// of the folder's entry names.
/// If it is a file, it will return the content of the file as a binary stream.
/// 
/// # Arguments
/// 
/// - `State(cfg)` - The app configuration as a shared state.
/// - `subpath` - The path to the resource as specified in the http route.
/// - `params` - Specify resizing options for images.
pub async fn download(
    State(cfg): State<Arc<AppState>>,
    subpath: Option<extract::Path<String>>,
    params: Query<Params>
) -> ApiResult<Response> {
    let subpath = subpath.as_ref().map(|p| p.as_str());
    let fullpath = make_fullpath(&cfg.root, subpath)?;
    let is_dir = is_dir(&fullpath).await?;

    let result: ApiResult<Response> = if is_dir {
        get_folder_entries(&fullpath).await
            .map(|children| Json(children).into_response())
    }
    else {
        get_file_stream(&fullpath).await
            .map(|stream| stream.into_response())
    };

    result
}

// https://docs.rs/image/latest/image/index.html

#[derive(Default, Deserialize)]
pub struct Params {
    max_width: Option<u32>,
    max_height: Option<u32>,
}

/// Makes a fullpath valid on the local file system from the path of
/// the http route.
fn make_fullpath(root: &str, subpath: Option<&str>) -> ApiResult<PathBuf>
{
    let mut fullpath = Path::new(root).to_path_buf();
    if let Some(subpath) = subpath {
        fullpath = fullpath.join(subpath);
    }

    if fullpath.exists() {
        Ok(fullpath)
    }
    else {
        let msg = format!("path {} doesn't exist", fullpath.to_str().unwrap_or(""));
        Err(ApiError::new(StatusCode::NOT_FOUND).with_msg(msg))
    }
}

/// Gets the mimetype of a file on the local file system.
fn get_mimetype (filepath : &PathBuf) -> Mime {
    mime_guess::from_path(filepath)
        .first()
        .unwrap_or(mime::APPLICATION_OCTET_STREAM)
}

/// Checks whether `path` is a directory.
async fn is_dir(path: &PathBuf) -> ApiResult<bool>
{
    let metadata = fs::metadata(path).await?;
    Ok(metadata.is_dir())
}

/// Returns the filename of all the entry in the folder specified by
/// `fullpath`
async fn get_folder_entries(fullpath: &PathBuf) -> ApiResult<Vec<String>> {
    let entries = fs::read_dir(fullpath).await?;
    let mut entries = ReadDirStream::new(entries);

    let mut result = vec![];
    while let Some(entry) = entries.next().await {
        if let Ok(entry) = entry {
            let filename = entry.file_name().to_str()
                .ok_or_else(||
                    ApiError::new(StatusCode::INTERNAL_SERVER_ERROR)
                    .with_msg("Encoding issue".to_string())
                )?
                .to_string();
            result.push(filename);
        }
    }

    Ok(result)
}

/// Returns the content of the file specified by `fullpath` as a binary
/// stream.
async fn get_file_stream(fullpath: &PathBuf) -> ApiResult<impl IntoResponse> {
    // Based on https://github.com/tokio-rs/axum/discussions/608

    // `File` implements `AsyncRead`
    let file = tokio::fs::File::open(fullpath).await?;

    let filename = fullpath.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let mimetype = get_mimetype(fullpath);

    let headers = [
        (
            header::CONTENT_TYPE,
            mimetype.to_string()
        ),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        ),
    ];

    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = StreamBody::new(stream);

    Ok((headers, body))
}

#[cfg(test)]
mod tests;
