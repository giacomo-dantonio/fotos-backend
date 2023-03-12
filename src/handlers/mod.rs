use crate::{
    api::error::{ApiError, ApiResult},
    AppState
};

use axum::{
    body::StreamBody,
    extract::{self, State},
    http::{StatusCode, header},
    Json,
    response::{IntoResponse, Response}
};
use mime::Mime;
use mime_guess;
use tokio_stream::StreamExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio_stream::wrappers::ReadDirStream;
use tokio_util::io::ReaderStream;

// TODO: add documentation

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

fn get_mimetype (filepath : &PathBuf) -> Mime {
    mime_guess::from_path(filepath)
        .first()
        .unwrap_or(mime::APPLICATION_OCTET_STREAM)
}

async fn is_dir(path: &PathBuf) -> ApiResult<bool>
{
    let metadata = fs::metadata(path).await?;
    Ok(metadata.is_dir())
}

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

pub async fn download(
    State(cfg): State<Arc<AppState>>,
    subpath: Option<extract::Path<String>>
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

#[cfg(test)]
mod tests;
