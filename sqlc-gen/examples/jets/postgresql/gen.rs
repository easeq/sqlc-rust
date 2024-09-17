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
}
const COUNT_PILOTS: &str = r#"
SELECT COUNT(*) FROM pilots
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct CountPilotsRow {
    pub(crate) count: i64,
}
impl Queries {
    pub fn count_pilots(
        &self,
        params: CountPilotsParams,
    ) -> anyhow::Result<CountPilotsRow> {
        let row: CountPilotsRow = self.client.query_one(COUNT_PILOTS, &[])?;
        Ok(row)
    }
}
const LIST_PILOTS: &str = r#"
SELECT id, name FROM pilots LIMIT 5
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct ListPilotsRow {
    pub(crate) id: i32,
    pub(crate) name: String,
}
impl Queries {
    pub fn list_pilots(
        &self,
        params: ListPilotsParams,
    ) -> anyhow::Result<Vec<ListPilotsRow>> {
        let rows = self.client.query(LIST_PILOTS, &[])?;
        let result: Vec<ListPilotsRow> = vec![];
        for row in rows {
            let row: ListPilotsRow = row.into();
            result.push(row);
        }
        Ok(result)
    }
}
const DELETE_PILOT: &str = r#"
DELETE FROM pilots WHERE id = $1
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct DeletePilotParams {
    pub(crate) id: i32,
}
impl Queries {
    pub fn delete_pilot(&self, params: DeletePilotParams) -> anyhow::Result<()> {
        self.client.execute(DELETE_PILOT, &[&params.id])?;
        Ok(())
    }
}
