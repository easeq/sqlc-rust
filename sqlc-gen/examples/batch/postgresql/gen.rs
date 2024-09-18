#[derive(Debug, Display)]
pub enum BookType {
    Fiction,
    Nonfiction,
}
const GET_AUTHOR: &str = r#"
select author_id, name, biography
from authors
where author_id = $1
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct GetAuthorParams {
    pub(crate) author_id: u16,
}
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct GetAuthorRow {
    pub(crate) author_id: u16,
    pub(crate) name: String,
    pub(crate) biography: Option<serde_json::Value>,
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
        let row: GetAuthorRow = self.client.query_one(GET_AUTHOR, &[&params.author_id])?;
        Ok(row)
    }
}
