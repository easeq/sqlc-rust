/// @generated by the sqlc-gen-rust on sqlc-generate using sqlc.yaml
/// DO NOT EDIT.
const GET_AUTHOR: &str = r#"
SELECT id, name, bio FROM authors
WHERE id = $1 LIMIT 1
"#;
const LIST_AUTHORS: &str = r#"
SELECT id, name, bio FROM authors
ORDER BY name
"#;
const CREATE_AUTHOR: &str = r#"
INSERT INTO authors (
          name, bio
) VALUES (
  $1, $2
)
RETURNING id, name, bio
"#;
const DELETE_AUTHOR: &str = r#"
DELETE FROM authors
WHERE id = $1
"#;
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
pub(crate) struct Author {
    pub id: i64,
    pub name: String,
    pub bio: Option<String>,
}
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
pub(crate) struct CreateAuthorParams {
    pub name: String,
    pub bio: Option<String>,
}
pub struct Queries {
    client: postgres::Client,
}
impl Queries {
    pub fn new(client: postgres::Client) -> Self {
        Self { client }
    }
    pub(crate) fn create_author(
        &mut self,
        arg: CreateAuthorParams,
    ) -> Result<Author, sqlc_core::Error> {
        let row = self.client.query_one(CREATE_AUTHOR, &[&arg.name, &arg.bio])?;
        Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
    }
    pub(crate) fn delete_author(&mut self, id: i64) -> Result<(), sqlc_core::Error> {
        self.client.execute(DELETE_AUTHOR, &[&id])?;
        Ok(())
    }
    pub(crate) fn get_author(&mut self, id: i64) -> Result<Author, sqlc_core::Error> {
        let row = self.client.query_one(GET_AUTHOR, &[&id])?;
        Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
    }
    pub(crate) fn list_authors(&mut self) -> Result<Vec<Author>, sqlc_core::Error> {
        let rows = self.client.query(LIST_AUTHORS, &[])?;
        let mut result: Vec<Author> = vec![];
        for row in rows {
            result.push(sqlc_core::FromPostgresRow::from_row(&row)?);
        }
        Ok(result)
    }
}
