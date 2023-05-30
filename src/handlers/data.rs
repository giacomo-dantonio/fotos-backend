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
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio_stream::wrappers::ReadDirStream;
use tokio_util::io::ReaderStream;

#[derive(Eq, PartialEq, PartialOrd, Debug, Ord, Serialize)]
struct FolderEntry {
    filename: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    mimetype: Option<String>,

    is_dir: bool
}

impl FolderEntry {
    async fn new(parent: &Path, filename: &str) -> ApiResult<Self> {
        let filepath = parent.join(filename);
        if is_dir(&filepath).await? {
            Ok(Self {
                filename: filename.to_string(),
                mimetype: None,
                is_dir: true
            })
        } else {
            Ok(Self {
                filename: filename.to_string(),
                mimetype: Some(get_mimetype(&filepath).to_string()),
                is_dir: false
            })
        }
    }
}

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
/// - `State(state)` - The shared state of the application.
/// - `subpath` - The path to the resource as specified in the http route.
/// - `params` - Specify resizing options for images.
pub async fn download(
    State(state): State<Arc<AppState>>,
    subpath: Option<extract::Path<String>>,
    params: Query<Params>
) -> ApiResult<Response> {
    let subpath = subpath.as_ref().map(|p| p.as_str());
    let fullpath = make_fullpath(&state.conf.root, subpath)?;
    let is_dir = is_dir(&fullpath).await?;

    let result: ApiResult<Response> = if is_dir {
        get_folder_entries(&fullpath).await
            .map(|children| Json(children).into_response())
    }
    else {
        get_file_stream(&fullpath, &params).await
            .map(|stream| stream.into_response())
    };

    result
}

/// Query parameters for the data endpoint.
/// 
/// - `max_width` - If provided will rescale the image such to have width
///     smaller than `max_width`
/// - `max_height` - If provided will rescale the image such to have height
///     smaller than `max_height`
/// - `thumbnails` - If provided and set to true a fast integer algorithm
///     will be used for resizing.
///     This May give aliasing artifacts if new size is close to old size.
#[derive(Default, Deserialize)]
pub struct Params {
    max_width: Option<u32>,
    max_height: Option<u32>,
    thumbnail: Option<bool>
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
async fn get_folder_entries(fullpath: &PathBuf) -> ApiResult<Vec<FolderEntry>> {
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

            result.push(FolderEntry::new(fullpath, &filename).await?);
        }
    }

    Ok(result)
}

/// Returns the content of the file specified by `fullpath` as a binary
/// stream.
async fn get_file_stream(fullpath: &PathBuf, params: &Params) -> ApiResult<impl IntoResponse> {
    // Based on https://github.com/tokio-rs/axum/discussions/608

    let resize = imgs::is_image(fullpath)
        && imgs::needs_resize(fullpath, params.max_width, params.max_height)?;
    let body: Response = if resize {
        let bytes = imgs::resize(
            fullpath,
            params.max_width,
            params.max_height,
            params.thumbnail.unwrap_or(false)
        ).await?;
        bytes.into_response()
    } else {
        let file = tokio::fs::File::open(fullpath).await?;
        let stream = ReaderStream::new(file);
        StreamBody::new(stream).into_response()
    };

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

    Ok((headers, body))
}

pub mod imgs;

#[cfg(test)]
mod tests;
