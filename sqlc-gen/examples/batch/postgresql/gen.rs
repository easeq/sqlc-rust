/// @generated by the sqlc-gen-rust on sqlc-generate using sqlc.yaml
/// DO NOT EDIT.
use postgres::{Error, Row};
const GET_AUTHOR: &str = r#"
select author_id, name, biography
from authors
where author_id = $1
"#;
#[derive(Debug, Display, postgres_types::ToSql, postgres_type::FromSql)]
pub enum BookType {
    #[postgres(name = "FICTION")]
    Fiction,
    #[postgres(name = "NONFICTION")]
    Nonfiction,
}
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
pub(crate) struct Author {
    pub author_id: i32,
    pub name: String,
    pub biography: Option<serde_json::Value>,
}
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
pub(crate) struct Book {
    pub book_id: i32,
    pub author_id: i32,
    pub isbn: String,
    pub book_type: String,
    pub title: String,
    pub year: i32,
    pub available: String,
    pub tags: Vec<String>,
}
pub struct Queries {
    client: postgres::Client,
}
impl Queries {
    pub fn new(client: postgres::Client) -> Self {
        Self { client }
    }
    pub(crate) fn get_author(&mut self, author_id: i32) -> anyhow::Result<Author> {
        let row = self.client.query_one(GET_AUTHOR, &[&author_id])?;
        Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
    }
}
