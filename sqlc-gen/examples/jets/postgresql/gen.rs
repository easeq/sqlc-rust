/// @generated by the sqlc-gen-rust on sqlc-generate using sqlc.yaml
/// DO NOT EDIT.
pub(crate) const COUNT_PILOTS: &str = r#"SELECT COUNT(*) FROM pilots"#;
pub(crate) const DELETE_PILOT: &str = r#"DELETE FROM pilots WHERE id = $1"#;
pub(crate) const LIST_PILOTS: &str = r#"SELECT id, name FROM pilots LIMIT 5"#;
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct Jet {
    pub id: i32,
    pub pilot_id: i32,
    pub age: i32,
    pub name: String,
    pub color: String,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct Language {
    pub id: i32,
    pub language: String,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct Pilot {
    pub id: i32,
    pub name: String,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct PilotLanguage {
    pub pilot_id: i32,
    pub language_id: i32,
}
pub(crate) async fn count_pilots(
    client: &impl sqlc_core::DBTX,
) -> sqlc_core::Result<i64> {
    let row = client.query_one(COUNT_PILOTS, &[]).await?;
    Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
}
pub(crate) async fn delete_pilot(
    client: &impl sqlc_core::DBTX,
    id: i32,
) -> sqlc_core::Result<()> {
    client.execute(DELETE_PILOT, &[&id]).await?;
    Ok(())
}
pub(crate) async fn list_pilots(
    client: &impl sqlc_core::DBTX,
) -> sqlc_core::Result<impl std::iter::Iterator<Item = sqlc_core::Result<Pilot>>> {
    let rows = client.query(LIST_PILOTS, &[]).await?;
    let iter = rows
        .into_iter()
        .map(|row| Ok(sqlc_core::FromPostgresRow::from_row(&row)?));
    Ok(iter)
}
