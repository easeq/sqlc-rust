/// @generated by the sqlc-gen-rust on sqlc-generate using sqlc.yaml
/// DO NOT EDIT.
pub(crate) const CREATE_AUTHOR: &str = r#"
INSERT INTO authors (
          name, bio
) VALUES (
  $1, $2
)
RETURNING id, name, bio
"#;
pub(crate) const DELETE_AUTHOR: &str = r#"
delete from authors
where id = $1
"#;
pub(crate) const GET_AUTHOR: &str = r#"
select id, name, bio
from authors
where id = $1
limit 1
"#;
pub(crate) const LIST_AUTHORS: &str = r#"
select id, name, bio
from authors
order by name
"#;
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct Author {
    pub id: i64,
    pub name: String,
    pub bio: Option<String>,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct CreateAuthorParams {
    pub name: String,
    pub bio: Option<String>,
}
pub(crate) async fn create_author(
    client: &impl sqlc_core::DBTX,
    arg: CreateAuthorParams,
) -> sqlc_core::Result<Author> {
    let row = client.query_one(CREATE_AUTHOR, &[&arg.name, &arg.bio]).await?;
    Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
}
pub(crate) async fn delete_author(
    client: &impl sqlc_core::DBTX,
    id: i64,
) -> sqlc_core::Result<()> {
    client.execute(DELETE_AUTHOR, &[&id]).await?;
    Ok(())
}
pub(crate) async fn get_author(
    client: &impl sqlc_core::DBTX,
    id: i64,
) -> sqlc_core::Result<Author> {
    let row = client.query_one(GET_AUTHOR, &[&id]).await?;
    Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
}
pub(crate) async fn list_authors(
    client: &impl sqlc_core::DBTX,
) -> sqlc_core::Result<impl std::iter::Iterator<Item = sqlc_core::Result<Author>>> {
    let rows = client.query(LIST_AUTHORS, &[]).await?;
    let iter = rows
        .into_iter()
        .map(|row| Ok(sqlc_core::FromPostgresRow::from_row(&row)?));
    Ok(iter)
}
