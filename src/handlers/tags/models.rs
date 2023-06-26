use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Clone, Deserialize, Debug, FromRow, Serialize)]
pub struct Tag {
    pub id: String,
    pub tagname: String
}
