use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, FromRow, Debug)]
pub struct Tag {
    pub id: Uuid,
    pub tagname: String
}
