use futures::StreamExt;
use futures::TryStreamExt;
use itertools::Itertools;
use std::ops::{Deref, DerefMut};

pub(crate) const ALL_BOOKS: &str = r#"
select book_id, author_id, isbn, book_type, title, year, available, tags
from books
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

#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct UpdateBookParams {
    pub title: String,
    pub tags: Vec<String>,
    pub book_id: i32,
}
pub(crate) async fn all_books(
    client: &impl sqlc_core::DBTX,
) -> sqlc_core::Result<impl std::iter::Iterator<Item = sqlc_core::Result<Book>>> {
    let rows = client.query(ALL_BOOKS, &[]).await?;
    let iter = rows
        .into_iter()
        .map(|row| Ok(sqlc_core::FromPostgresRow::from_row(&row)?));
    Ok(iter)
}
pub(crate) async fn create_author(
    client: &impl sqlc_core::DBTX,
    name: &str,
) -> sqlc_core::Result<Author> {
    let row = client.query_one(CREATE_AUTHOR, &[&name]).await?;
    Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
}

pub(crate) async fn create_book<'a, T: sqlc_core::DBTX>(
    client: &'a T,
    arg_list: &'a [CreateBookParams],
) -> sqlc_core::Result<
    impl futures::Stream<Item = impl futures::Future<Output = sqlc_core::Result<Book>> + 'a> + 'a,
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

pub(crate) async fn update_book<'a, C, I>(
    client: &'a C,
    arg_list: I,
) -> sqlc_core::Result<
    impl futures::Stream<Item = impl futures::Future<Output = sqlc_core::Result<()>> + 'a> + 'a,
>
where
    C: sqlc_core::DBTX,
    I: IntoIterator + 'a,
    I::Item: std::borrow::Borrow<UpdateBookParams> + 'a,
{
    use std::borrow::Borrow;
    let stmt = client.prepare(UPDATE_BOOK).await?;
    let fut = move |item: <I as IntoIterator>::Item| {
        let stmt = stmt.clone();
        Box::pin(async move {
            let arg = item.borrow();
            client
                .execute(&stmt, &[&arg.title, &arg.tags, &arg.book_id])
                .await?;
            Ok(())
        })
    };
    Ok(futures::stream::iter(arg_list.into_iter().map(fut)))
}

pub(crate) async fn books_by_year<'a, T: sqlc_core::DBTX>(
    client: &'a T,
    year_list: impl std::iter::Iterator<Item = i32> + 'a,
) -> sqlc_core::Result<
    impl futures::Stream<
            Item = impl futures::Future<
                Output = sqlc_core::Result<
                    impl futures::Stream<Item = sqlc_core::Result<sqlc_core::Result<Book>>>,
                >,
            > + 'a,
        > + 'a,
> {
    let stmt = client.prepare(BOOKS_BY_YEAR).await?;
    let fut = move |year: i32| {
        let stmt = stmt.clone();
        Box::pin(async move {
            let rows = client.query(&stmt, &[&year]).await?;
            let result = rows
                .into_iter()
                .map(|row| Ok(sqlc_core::FromPostgresRow::from_row(&row)));
            Ok(Box::pin(futures::stream::iter(result)))
        })
    };
    Ok(futures::stream::iter(year_list.map(fut)))
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

    println!("books: {:#?}", new_books);
    assert_eq!(new_books.len(), new_book_params.len());

    let mut db_client = pool.get().await.expect("failed to get client from pool");
    let client = db_client.deref_mut().deref_mut();

    let transaction = client
        .transaction()
        .await
        .expect("could not create transaction");

    // let update_books_params = vec![
    //     UpdateBookParams {
    //         book_id: new_books[1].book_id,
    //         title: "changed second txn title".to_string(),
    //         tags: vec!["cool".to_string(), "disastor".to_string()],
    //     },
    //     UpdateBookParams {
    //         book_id: new_books[2].book_id,
    //         title: "changed third txn title".to_string(),
    //         tags: vec!["cool".to_string(), "disastor".to_string()],
    //     },
    // ];

    let update_new_books_iter = new_books.iter().filter_map(|book| {
        if book.book_id % 2 == 0 {
            None
        } else {
            Some(UpdateBookParams {
                book_id: book.book_id,
                title: format!("{} updated", book.title),
                tags: book.tags.clone(),
            })
        }
    });

    update_book(&transaction, update_new_books_iter)
        .await
        .expect("failed to create update books results")
        .buffer_unordered(1)
        .try_collect::<Vec<_>>()
        .await
        .expect("failed to update books");

    transaction
        .commit()
        .await
        .expect("failed to commit transaction");

    let books: Vec<_> = all_books(client)
        .await
        .expect("failed to fetch all books")
        .try_collect()
        .expect("failed to collect all books");
    println!("{books:#?}");
}
