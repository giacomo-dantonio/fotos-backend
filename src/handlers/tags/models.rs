use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Clone, Deserialize, Debug, FromRow, Serialize)]
pub struct Tag {
    pub id: String,
    pub tagname: String
}

#[derive(Clone, Deserialize, Debug, FromRow, Serialize)]
pub struct File {
    pub id: String,
    pub relative_path: String,
    pub csum: String
}

#[derive(Clone, Deserialize, Debug, FromRow, Serialize)]
pub struct FileTags {
    pub tag_id: String,
    pub file_id: String
}
