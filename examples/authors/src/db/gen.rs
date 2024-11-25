/// @generated by the sqlc-gen-rust on sqlc-generate using sqlc.yaml
/// DO NOT EDIT.
pub(crate) const CREATE_AUTHOR: &str = r#"
INSERT INTO authors (
  name, bio
) VALUES (
  $1, $2
)
RETURNING id, uuid, name, genre, bio, data, attrs, ip_inet, ip_cidr, mac_address, geo_point, geo_rect, geo_path, bit_a, varbit_a, created_at, updated_at
"#;
pub(crate) const CREATE_AUTHOR_FULL: &str = r#"
INSERT INTO authors (
  name, 
  bio,
  data,
  genre,
  attrs,
  ip_inet,
  ip_cidr,
  mac_address,
  geo_point,
  geo_rect,
  geo_path,
  bit_a,
  varbit_a,
  created_at,
  updated_at
) VALUES (
  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
)
RETURNING id, uuid, name, genre, bio, data, attrs, ip_inet, ip_cidr, mac_address, geo_point, geo_rect, geo_path, bit_a, varbit_a, created_at, updated_at
"#;
pub(crate) const DELETE_AUTHOR: &str = r#"
delete from authors
where id = $1
"#;
pub(crate) const GET_AUTHOR: &str = r#"
select id, uuid, name, genre, bio, data, attrs, ip_inet, ip_cidr, mac_address, geo_point, geo_rect, geo_path, bit_a, varbit_a, created_at, updated_at
from authors
where id = $1
limit 1
"#;
pub(crate) const LIST_AUTHORS: &str = r#"
select id, uuid, name, genre, bio, data, attrs, ip_inet, ip_cidr, mac_address, geo_point, geo_rect, geo_path, bit_a, varbit_a, created_at, updated_at
from authors
order by name
"#;
#[derive(Clone, Debug, PartialEq, postgres_derive::ToSql, postgres_derive::FromSql)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
#[postgres(name = "type_genre")]
pub enum TypeGenre {
    #[postgres(name = "history")]
    #[cfg_attr(feature = "serde_support", serde(rename = "history"))]
    History,
    #[postgres(name = "Children")]
    #[cfg_attr(feature = "serde_support", serde(rename = "Children"))]
    Children,
    #[postgres(name = "cLaSSic")]
    #[cfg_attr(feature = "serde_support", serde(rename = "cLaSSic"))]
    CLaSSic,
    #[postgres(name = "ADVENTURE")]
    #[cfg_attr(feature = "serde_support", serde(rename = "ADVENTURE"))]
    Adventure,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct Author {
    pub id: i64,
    pub uuid: Option<uuid::Uuid>,
    pub name: String,
    pub genre: TypeGenre,
    pub bio: Option<String>,
    pub data: Option<serde_json::Value>,
    pub attrs: Option<std::collections::HashMap<String, Option<String>>>,
    pub ip_inet: cidr::IpInet,
    pub ip_cidr: cidr::IpCidr,
    pub mac_address: eui48::MacAddress,
    pub geo_point: Option<geo_types::Point<f64>>,
    pub geo_rect: Option<geo_types::Rect<f64>>,
    pub geo_path: Option<geo_types::LineString<f64>>,
    pub bit_a: Option<bit_vec::BitVec>,
    pub varbit_a: Option<bit_vec::BitVec>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct CreateAuthorFullParams {
    pub name: String,
    pub bio: Option<String>,
    pub data: Option<serde_json::Value>,
    pub genre: TypeGenre,
    pub attrs: Option<std::collections::HashMap<String, Option<String>>>,
    pub ip_inet: cidr::IpInet,
    pub ip_cidr: cidr::IpCidr,
    pub mac_address: eui48::MacAddress,
    pub geo_point: Option<geo_types::Point<f64>>,
    pub geo_rect: Option<geo_types::Rect<f64>>,
    pub geo_path: Option<geo_types::LineString<f64>>,
    pub bit_a: Option<bit_vec::BitVec>,
    pub varbit_a: Option<bit_vec::BitVec>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct CreateAuthorParams {
    pub name: String,
    pub bio: Option<String>,
}
pub(crate) fn create_author(
    client: &impl sqlc_core::DBTX,
    arg: CreateAuthorParams,
) -> Result<Author, sqlc_core::Error> {
    let row = client.query_one(CREATE_AUTHOR, &[&arg.name, &arg.bio])?;
    Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
}
pub(crate) fn create_author_full(
    client: &impl sqlc_core::DBTX,
    arg: CreateAuthorFullParams,
) -> Result<Author, sqlc_core::Error> {
    let row = client
        .query_one(
            CREATE_AUTHOR_FULL,
            &[
                &arg.name,
                &arg.bio,
                &arg.data,
                &arg.genre,
                &arg.attrs,
                &arg.ip_inet,
                &arg.ip_cidr,
                &arg.mac_address,
                &arg.geo_point,
                &arg.geo_rect,
                &arg.geo_path,
                &arg.bit_a,
                &arg.varbit_a,
                &arg.created_at,
                &arg.updated_at,
            ],
        )?;
    Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
}
pub(crate) fn delete_author(
    client: &impl sqlc_core::DBTX,
    id: i64,
) -> Result<(), sqlc_core::Error> {
    client.execute(DELETE_AUTHOR, &[&id])?;
    Ok(())
}
pub(crate) fn get_author(
    client: &impl sqlc_core::DBTX,
    id: i64,
) -> Result<Author, sqlc_core::Error> {
    let row = client.query_one(GET_AUTHOR, &[&id])?;
    Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
}
pub(crate) fn list_authors(
    client: &impl sqlc_core::DBTX,
) -> Result<Vec<Author>, sqlc_core::Error> {
    let rows = client.query(LIST_AUTHORS, &[])?;
    let mut result: Vec<Author> = vec![];
    for row in rows {
        result.push(sqlc_core::FromPostgresRow::from_row(&row)?);
    }
    Ok(result)
}
