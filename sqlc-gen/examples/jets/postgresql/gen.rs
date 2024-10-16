/// @generated by the sqlc-gen-rust on sqlc-generate using sqlc.yaml
/// DO NOT EDIT.
const COUNT_PILOTS: &str = r#"SELECT COUNT(*) FROM pilots"#;
const LIST_PILOTS: &str = r#"SELECT id, name FROM pilots LIMIT 5"#;
const DELETE_PILOT: &str = r#"DELETE FROM pilots WHERE id = $1"#;
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
pub(crate) struct Jet {
    pub id: i32,
    pub pilot_id: i32,
    pub age: i32,
    pub name: String,
    pub color: String,
}
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
pub(crate) struct Language {
    pub id: i32,
    pub language: String,
}
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
pub(crate) struct Pilot {
    pub id: i32,
    pub name: String,
}
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
pub(crate) struct PilotLanguage {
    pub pilot_id: i32,
    pub language_id: i32,
}
pub struct Queries {
    client: tokio_postgres::Client,
}
impl Queries {
    pub fn new(client: tokio_postgres::Client) -> Self {
        Self { client }
    }
    pub(crate) async fn count_pilots(&mut self) -> Result<i64, sqlc_core::Error> {
        let row = self.client.query_one(COUNT_PILOTS, &[]).await?;
        Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
    }
    pub(crate) async fn delete_pilot(
        &mut self,
        id: i32,
    ) -> Result<(), sqlc_core::Error> {
        self.client.execute(DELETE_PILOT, &[&id]).await?;
        Ok(())
    }
    pub(crate) async fn list_pilots(&mut self) -> Result<Vec<Pilot>, sqlc_core::Error> {
        let rows = self.client.query(LIST_PILOTS, &[]).await?;
        let mut result: Vec<Pilot> = vec![];
        for row in rows {
            result.push(sqlc_core::FromPostgresRow::from_row(&row)?);
        }
        Ok(result)
    }
}
