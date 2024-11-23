/// @generated by the sqlc-gen-rust on sqlc-generate using sqlc.yaml
/// DO NOT EDIT.
pub(crate) const GET_AUTHOR: &str = r#"
select author_id, name, biography
from authors
where author_id = $1
"#;
pub(crate) const DELETE_BOOK_EXEC_RESULT: &str = r#"
delete from books
where book_id = $1
"#;
pub(crate) const DELETE_BOOK: &str = r#"
delete from books
where book_id = $1
"#;
pub(crate) const DELETE_BOOK_NAMED_FUNC: &str = r#"
delete from books
where book_id = $1
"#;
pub(crate) const DELETE_BOOK_NAMED_SIGN: &str = r#"
delete from books
where book_id = $1
"#;
pub(crate) const BOOKS_BY_YEAR: &str = r#"
select book_id, author_id, isbn, book_type, title, year, available, tags
from books
where year = $1
"#;
pub(crate) const CREATE_AUTHOR: &str = r#"
INSERT INTO authors (name) VALUES ($1)
RETURNING author_id, name, biography
"#;
pub(crate) const CREATE_BOOK: &str = r#"
INSERT INTO books (
  author_id,
  isbn,
  book_type,
  title,
  year,
  available,
  tags
) VALUES (
  $1,
  $2,
  $3,
  $4,
  $5,
  $6,
  $7
)
RETURNING book_id, author_id, isbn, book_type, title, year, available, tags
"#;
pub(crate) const UPDATE_BOOK: &str = r#"
UPDATE books
SET title = $1, tags = $2
WHERE book_id = $3
"#;
pub(crate) const GET_BIOGRAPHY: &str = r#"
select biography
from authors
where author_id = $1
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
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct CreateBookParams {
    pub author_id: i32,
    pub isbn: String,
    pub book_type: BookType,
    pub title: String,
    pub year: i32,
    pub available: time::OffsetDateTime,
    pub tags: Vec<String>,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct UpdateBookParams {
    pub title: String,
    pub tags: Vec<String>,
    pub book_id: i32,
}
pub(crate) async fn books_by_year<'a, 'b, T: sqlc_core::DBTX>(
    client: &'a T,
    year_list: &'b [i32],
) -> Result<
    impl futures::Stream<
        Item = std::pin::Pin<
            Box<
                impl futures::Future<
                    Output = Result<
                        std::pin::Pin<
                            Box<
                                futures::stream::Iter<
                                    std::vec::IntoIter<Result<Book, sqlc_core::Error>>,
                                >,
                            >,
                        >,
                        sqlc_core::Error,
                    >,
                > + use<'a, 'b, T>,
            >,
        >,
    > + use<'a, 'b, T>,
    sqlc_core::Error,
> {
    let stmt = client.prepare(BOOKS_BY_YEAR).await?;
    let mut futs = vec![];
    for year in year_list {
        let stmt = stmt.clone();
        futs.push(
            Box::pin(async move {
                let rows = client.query(&stmt, &[&year]).await?;
                let mut result: Vec<Result<Book, sqlc_core::Error>> = vec![];
                for row in rows {
                    result.push(Ok(sqlc_core::FromPostgresRow::from_row(&row)?));
                }
                Ok(Box::pin(futures::stream::iter(result)))
            }),
        );
    }
    Ok(futures::stream::iter(futs))
}
pub(crate) async fn create_author(
    client: &impl sqlc_core::DBTX,
    name: String,
) -> Result<Author, sqlc_core::Error> {
    let row = client.query_one(CREATE_AUTHOR, &[&name]).await?;
    Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
}
pub(crate) async fn create_book<'a, 'b, T: sqlc_core::DBTX>(
    client: &'a T,
    arg_list: &'b [CreateBookParams],
) -> Result<
    impl futures::Stream<
        Item = std::pin::Pin<
            Box<
                impl futures::Future<
                    Output = Result<Book, sqlc_core::Error>,
                > + use<'a, 'b, T>,
            >,
        >,
    > + use<'a, 'b, T>,
    sqlc_core::Error,
> {
    let stmt = client.prepare(CREATE_BOOK).await?;
    let mut futs = vec![];
    for arg in arg_list {
        let stmt = stmt.clone();
        futs.push(
            Box::pin(async move {
                let row = client
                    .query_one(
                        &stmt,
                        &[
                            &arg.author_id,
                            &arg.isbn,
                            &arg.book_type,
                            &arg.title,
                            &arg.year,
                            &arg.available,
                            &arg.tags,
                        ],
                    )
                    .await?;
                Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
            }),
        );
    }
    Ok(futures::stream::iter(futs))
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
pub(crate) async fn delete_book_exec_result(
    client: &impl sqlc_core::DBTX,
    book_id: i32,
) -> Result<(), sqlc_core::Error> {
    client.execute(DELETE_BOOK_EXEC_RESULT, &[&book_id]).await?;
    Ok(())
}
pub(crate) async fn delete_book_named_func<'a, 'b, T: sqlc_core::DBTX>(
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
    let stmt = client.prepare(DELETE_BOOK_NAMED_FUNC).await?;
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
pub(crate) async fn delete_book_named_sign<'a, 'b, T: sqlc_core::DBTX>(
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
    let stmt = client.prepare(DELETE_BOOK_NAMED_SIGN).await?;
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
pub(crate) async fn get_author(
    client: &impl sqlc_core::DBTX,
    author_id: i32,
) -> Result<Author, sqlc_core::Error> {
    let row = client.query_one(GET_AUTHOR, &[&author_id]).await?;
    Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
}
pub(crate) async fn get_biography<'a, 'b, T: sqlc_core::DBTX>(
    client: &'a T,
    author_id_list: &'b [i32],
) -> Result<
    impl futures::Stream<
        Item = std::pin::Pin<
            Box<
                impl futures::Future<
                    Output = Result<serde_json::Value, sqlc_core::Error>,
                > + use<'a, 'b, T>,
            >,
        >,
    > + use<'a, 'b, T>,
    sqlc_core::Error,
> {
    let stmt = client.prepare(GET_BIOGRAPHY).await?;
    let mut futs = vec![];
    for author_id in author_id_list {
        let stmt = stmt.clone();
        futs.push(
            Box::pin(async move {
                let row = client.query_one(&stmt, &[&author_id]).await?;
                Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
            }),
        );
    }
    Ok(futures::stream::iter(futs))
}
pub(crate) async fn update_book<'a, 'b, T: sqlc_core::DBTX>(
    client: &'a T,
    arg_list: &'b [UpdateBookParams],
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
    let stmt = client.prepare(UPDATE_BOOK).await?;
    let mut futs = vec![];
    for arg in arg_list {
        let stmt = stmt.clone();
        futs.push(
            Box::pin(async move {
                client.execute(&stmt, &[&arg.title, &arg.tags, &arg.book_id]).await?;
                Ok(())
            }),
        );
    }
    Ok(futures::stream::iter(futs))
}
