/// @generated by the sqlc-gen-rust on sqlc-generate using sqlc.yaml
/// DO NOT EDIT.
pub(crate) const DELETE_BOOK: &str = r#"
delete from books
where book_id = $1
"#;
#[derive(Clone, Debug, PartialEq, postgres_derive::ToSql, postgres_derive::FromSql)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
#[postgres(name = "book_type")]
pub enum BookType {
    #[postgres(name = "FICTION")]
    #[cfg_attr(feature = "serde_support", serde(rename = "FICTION"))]
    Fiction,
    #[postgres(name = "NONFICTION")]
    #[cfg_attr(feature = "serde_support", serde(rename = "NONFICTION"))]
    Nonfiction,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct Author {
    pub author_id: i32,
    pub name: String,
    pub biography: Option<serde_json::Value>,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct Book {
    pub book_id: i32,
    pub author_id: i32,
    pub isbn: String,
    pub book_type: BookType,
    pub title: String,
    pub year: i32,
    pub available: time::OffsetDateTime,
    pub tags: Vec<String>,
}
pub(crate) async fn delete_book<'a, 'b, T: sqlc_core::DBTX>(
    client: &'a T,
    book_id_list: &'b [i32],
) -> Result<
    impl futures::Stream<
        Item = std::pin::Pin<
            Box<
                impl futures::Future<
                    Output = Result<(), sqlc_core::Error>,
                > + use<'a, 'b, T>,
            >,
        >,
    > + use<'a, 'b, T>,
    sqlc_core::Error,
> {
    let stmt = client.prepare(DELETE_BOOK).await?;
    let mut futs = vec![];
    for book_id in book_id_list {
        let stmt = stmt.clone();
        futs.push(
            Box::pin(async move {
                client.execute(&stmt, &[&book_id]).await?;
                Ok(())
            }),
        );
    }
    Ok(futures::stream::iter(futs))
}
