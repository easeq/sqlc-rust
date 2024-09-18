const GET_AUTHOR: &str = r#"
SELECT id, name, bio FROM authors
WHERE id = $1 LIMIT 1
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct GetAuthorParams {
    pub(crate) id: u64,
}
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct GetAuthorRow {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) bio: Option<String>,
}
const LIST_AUTHORS: &str = r#"
SELECT id, name, bio FROM authors
ORDER BY name
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct ListAuthorsRow {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) bio: Option<String>,
}
const CREATE_AUTHOR: &str = r#"
INSERT INTO authors (
          name, bio
) VALUES (
  $1, $2
)
RETURNING id, name, bio
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct CreateAuthorParams {
    pub(crate) name: String,
    pub(crate) bio: Option<String>,
}
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct CreateAuthorRow {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) bio: Option<String>,
}
const DELETE_AUTHOR: &str = r#"
DELETE FROM authors
WHERE id = $1
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct DeleteAuthorParams {
    pub(crate) id: u64,
}
pub struct Queries {
    client: postgres::Client,
}
impl Queries {
    pub fn new(
        host: &str,
        port: &str,
        username: &str,
        password: &str,
        database_name: &str,
    ) -> Self {
        let client = postgres::Client::connect(
                format!(
                    "postgresql://{username}:{password}@{host}:{port}/{database_name}",
                    host = settings.host, port = settings.port, username = settings
                    .username, password = settings.password, database_name =
                    database_name,
                )
                    .as_str(),
                postgres::NoTls,
            )
            .unwrap();
        Self { client }
    }
    pub fn get_author(&self, params: GetAuthorParams) -> anyhow::Result<GetAuthorRow> {
        let row: GetAuthorRow = self.client.query_one(GET_AUTHOR, &[&params.id])?;
        Ok(row)
    }
    pub fn list_authors(
        &self,
        params: ListAuthorsParams,
    ) -> anyhow::Result<Vec<ListAuthorsRow>> {
        let rows = self.client.query(LIST_AUTHORS, &[])?;
        let result: Vec<ListAuthorsRow> = vec![];
        for row in rows {
            result.push(row.into());
        }
        Ok(result)
    }
    pub fn create_author(
        &self,
        params: CreateAuthorParams,
    ) -> anyhow::Result<CreateAuthorRow> {
        let row: CreateAuthorRow = self
            .client
            .query_one(CREATE_AUTHOR, &[&params.name, &params.bio])?;
        Ok(row)
    }
    pub fn delete_author(&self, params: DeleteAuthorParams) -> anyhow::Result<()> {
        self.client.execute(DELETE_AUTHOR, &[&params.id])?;
        Ok(())
    }
}
