/// @generated by the sqlc-gen-rust on sqlc-generate using sqlc.yaml
/// DO NOT EDIT.
const GET_AUTHOR: &str = r#"
select id, name, bio
from authors
where id = $1
limit 1
"#;
const LIST_AUTHORS: &str = r#"
select id, name, bio
from authors
order by name
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
delete from authors
where id = $1
"#;
const GET_SITE: &str = r#"
select id, url, status, data, inet, mac, new_id, created_at, updated_at
from site
where id = $1
limit 1
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
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
pub(crate) struct Site {
    pub id: uuid::Uuid,
    pub url: String,
    pub status: bool,
    pub data: serde_json::Value,
    pub inet: cidr::InetCidr,
    pub mac: eui48::MacAddress,
    pub new_id: uuid::Uuid,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}
pub struct Queries {
    client: tokio_postgres::Client,
}
impl Queries {
    pub fn new(client: tokio_postgres::Client) -> Self {
        Self { client }
    }
    pub(crate) async fn create_author(
        &mut self,
        arg: CreateAuthorParams,
    ) -> Result<Author, sqlc_core::Error> {
        let row = self.client.query_one(CREATE_AUTHOR, &[&arg.name, &arg.bio]).await?;
        Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
    }
    pub(crate) async fn delete_author(
        &mut self,
        id: i64,
    ) -> Result<(), sqlc_core::Error> {
        self.client.execute(DELETE_AUTHOR, &[&id]).await?;
        Ok(())
    }
    pub(crate) async fn get_author(
        &mut self,
        id: i64,
    ) -> Result<Author, sqlc_core::Error> {
        let row = self.client.query_one(GET_AUTHOR, &[&id]).await?;
        Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
    }
    pub(crate) async fn get_site(
        &mut self,
        id: uuid::Uuid,
    ) -> Result<Site, sqlc_core::Error> {
        let row = self.client.query_one(GET_SITE, &[&id]).await?;
        Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
    }
    pub(crate) async fn list_authors(
        &mut self,
    ) -> Result<Vec<Author>, sqlc_core::Error> {
        let rows = self.client.query(LIST_AUTHORS, &[]).await?;
        let mut result: Vec<Author> = vec![];
        for row in rows {
            result.push(sqlc_core::FromPostgresRow::from_row(&row)?);
        }
        Ok(result)
    }
}
