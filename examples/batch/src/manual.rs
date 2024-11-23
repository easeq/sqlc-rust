use futures::StreamExt;
use futures::TryStreamExt;
use std::ops::Deref;

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

#[derive(Clone, Debug, PartialEq, postgres_derive::ToSql, postgres_derive::FromSql)]
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
#[postgres(name = "book_type")]
pub enum BookType {
    #[postgres(name = "FICTION")]
    #[cfg_attr(feature = "serde_support", serde(rename = "FICTION"))]
    Fiction,
    #[postgres(name = "NONFICTION")]
    #[cfg_attr(feature = "serde_support", serde(rename = "NONFICTION"))]
    Nonfiction,
    // #[postgres(name = "INVALID")]
    // #[cfg_attr(feature = "serde_support", serde(rename = "INVALID"))]
    // Invalid,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
pub(crate) struct Author {
    pub author_id: i32,
    pub name: String,
    pub biography: Option<serde_json::Value>,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
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
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
pub(crate) struct CreateBookParams {
    pub author_id: i32,
    pub isbn: String,
    pub book_type: BookType,
    pub title: String,
    pub year: i32,
    pub available: time::OffsetDateTime,
    pub tags: Vec<String>,
}

pub(crate) async fn create_author(
    client: &impl sqlc_core::DBTX,
    name: &str,
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
                Box<impl futures::Future<Output = Result<Book, sqlc_core::Error>> + use<'a, 'b, T>>,
            >,
        > + use<'a, 'b, T>,
    sqlc_core::Error,
> {
    let stmt = client.prepare(CREATE_BOOK).await?;
    Ok(futures::stream::iter(arg_list.iter().map(move |arg| {
        let stmt = stmt.clone();
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
            let result: Book = sqlc_core::FromPostgresRow::from_row(&row)?;
            Ok::<Book, sqlc_core::Error>(result)
        })
    })))
}

pub(crate) async fn execute(pool: deadpool_postgres::Pool) {
    let db_client = pool.get().await.expect("failed to get client from pool");
    let client = db_client.deref().deref();

    let a = create_author(client, "Unknown Master").await.unwrap();

    let new_book_params = vec![
        CreateBookParams {
            author_id: a.author_id,
            isbn: "1".to_string(),
            title: "my book title".to_string(),
            book_type: BookType::Fiction,
            year: 2016,
            available: time::OffsetDateTime::now_utc(),
            tags: vec![],
        },
        CreateBookParams {
            author_id: a.author_id,
            isbn: "2".to_string(),
            title: "the second book".to_string(),
            book_type: BookType::Fiction,
            year: 2016,
            available: time::OffsetDateTime::now_utc(),
            tags: vec!["cool".to_string(), "unique".to_string()],
        },
        CreateBookParams {
            author_id: a.author_id,
            isbn: "3".to_string(),
            title: "the third book".to_string(),
            book_type: BookType::Nonfiction,
            year: 2001,
            available: time::OffsetDateTime::now_utc(),
            tags: vec!["cool".to_string()],
        },
        CreateBookParams {
            author_id: a.author_id,
            isbn: "4".to_string(),
            title: "4th place finisher".to_string(),
            book_type: BookType::Fiction,
            year: 2011,
            available: time::OffsetDateTime::now_utc(),
            tags: vec!["other".to_string()],
        },
    ];

    let new_books = create_book(client, &new_book_params)
        .await
        .expect("failed to create batch results")
        // .take(2)
        .buffer_unordered(10)
        .try_collect::<Vec<_>>()
        .await
        .expect("failed to collect batch results 1");

    println!("books: {:?}", new_books);
    assert_eq!(new_books.len(), new_book_params.len());
}
