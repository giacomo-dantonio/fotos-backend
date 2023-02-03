use crate::AppState;

use anyhow::{anyhow, Result};
use axum::Json;
use axum::extract::{self, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tokio_stream::StreamExt;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio_stream::wrappers::ReadDirStream;

async fn get_children(root: &str, subpath: Option<&str>) -> Result<Vec<String>>
{
    let mut fullpath = Path::new(root).to_path_buf();
    if let Some(subpath) = subpath {
        fullpath = fullpath.join(subpath);
    }

    if fullpath.exists() {
        let entries = fs::read_dir(&fullpath).await?;
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
    else {
        Err(anyhow!("path {} doesn't exist", fullpath.to_str().unwrap_or("")))
    }
}

pub async fn folder_list(
    State(cfg): State<Arc<AppState>>,
    subpath: Option<extract::Path<String>>) -> Result<Response, StatusCode> {

    let subpath = subpath.as_ref().map(|p| p.as_str());
    let children = get_children(&cfg.root, subpath).await;
    match children {
        Ok(children) => Ok(Json(children).into_response()),
        Err(_) => Err(StatusCode::BAD_REQUEST)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::fs;

    #[tokio::test]
    async fn get_children_test() {
        let root = Path::new(".")
            .join(std::file!())
            .parent().unwrap()
            .join("data");
        println!("{}", std::file!());
        println!("{:?}", root);
        println!("{}", fs::canonicalize(".").unwrap().to_str().unwrap());
        let root = fs::canonicalize(root).unwrap();

        let mut actual = super::get_children(root.to_str().unwrap(), None)
            .await.unwrap();
        actual.sort();

        let expected = vec![
            "apollon.jpg",
            "folder",
            "penguins.jpg"
        ];

        assert_eq!(actual, expected);
    }
}
