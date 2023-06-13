use sqlx::FromRow;

#[derive(Clone, FromRow, Debug)]
pub struct Tag {
    pub id: String,
    pub tagname: String
}
