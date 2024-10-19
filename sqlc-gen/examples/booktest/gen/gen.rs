/// @generated by the sqlc-gen-rust on sqlc-generate using sqlc.yaml
/// DO NOT EDIT.
const GET_AUTHOR: &str = r#"
select author_id, name
from authors
where author_id = $1
"#;
const GET_BOOK: &str = r#"
select book_id, author_id, isbn, book_type, title, year, available, tags
from books
where book_id = $1
"#;
const DELETE_BOOK: &str = r#"
delete from books
where book_id = $1
"#;
const BOOKS_BY_TITLE_YEAR: &str = r#"
select book_id, author_id, isbn, book_type, title, year, available, tags
from books
where title = $1 and year = $2
"#;
const BOOKS_BY_TAGS: &str = r#"
select book_id, title, name, isbn, tags
from books
left join authors on books.author_id = authors.author_id
where tags && $1::varchar[]
"#;
const CREATE_AUTHOR: &str = r#"
INSERT INTO authors (name) VALUES ($1)
RETURNING author_id, name
"#;
const CREATE_BOOK: &str = r#"
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
const UPDATE_BOOK: &str = r#"
UPDATE books
SET title = $1, tags = $2
WHERE book_id = $3
"#;
const UPDATE_BOOK_ISBN: &str = r#"
UPDATE books
SET title = $1, tags = $2, isbn = $4
WHERE book_id = $3
"#;
const SAY_HELLO: &str = r#"
select say_hello
from say_hello($1)
"#;
#[derive(Clone, Debug, PartialEq, postgres_derive::ToSql, postgres_derive::FromSql)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[postgres(name = "book_type")]
pub enum BookType {
    #[postgres(name = "FICTION")]
    #[cfg_attr(feature = "serde_support", serde(rename = "FICTION"))]
    Fiction,
    #[postgres(name = "NONFICTION")]
    #[cfg_attr(feature = "serde_support", serde(rename = "NONFICTION"))]
    Nonfiction,
}
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct Author {
    pub author_id: i32,
    pub name: String,
}
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
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
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct BooksByTagsRow {
    pub book_id: i32,
    pub title: String,
    pub name: Option<String>,
    pub isbn: String,
    pub tags: Vec<String>,
}
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct BooksByTitleYearParams {
    pub title: String,
    pub year: i32,
}
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct CreateBookParams {
    pub author_id: i32,
    pub isbn: String,
    pub book_type: BookType,
    pub title: String,
    pub year: i32,
    pub available: time::OffsetDateTime,
    pub tags: Vec<String>,
}
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct UpdateBookIsbnParams {
    pub title: String,
    pub tags: Vec<String>,
    pub book_id: i32,
    pub isbn: String,
}
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct UpdateBookParams {
    pub title: String,
    pub tags: Vec<String>,
    pub book_id: i32,
}
pub struct Queries {
    client: tokio_postgres::Client,
}
impl Queries {
    pub fn new(client: tokio_postgres::Client) -> Self {
        Self { client }
    }
    pub(crate) async fn books_by_tags(
        &mut self,
        dollar_1: String,
    ) -> Result<Vec<BooksByTagsRow>, sqlc_core::Error> {
        let rows = self.client.query(BOOKS_BY_TAGS, &[&dollar_1]).await?;
        let mut result: Vec<BooksByTagsRow> = vec![];
        for row in rows {
            result.push(sqlc_core::FromPostgresRow::from_row(&row)?);
        }
        Ok(result)
    }
    pub(crate) async fn books_by_title_year(
        &mut self,
        arg: BooksByTitleYearParams,
    ) -> Result<Vec<Book>, sqlc_core::Error> {
        let rows = self
            .client
            .query(BOOKS_BY_TITLE_YEAR, &[&arg.title, &arg.year])
            .await?;
        let mut result: Vec<Book> = vec![];
        for row in rows {
            result.push(sqlc_core::FromPostgresRow::from_row(&row)?);
        }
        Ok(result)
    }
    pub(crate) async fn create_author(
        &mut self,
        name: String,
    ) -> Result<Author, sqlc_core::Error> {
        let row = self.client.query_one(CREATE_AUTHOR, &[&name]).await?;
        Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
    }
    pub(crate) async fn create_book(
        &mut self,
        arg: CreateBookParams,
    ) -> Result<Book, sqlc_core::Error> {
        let row = self
            .client
            .query_one(
                CREATE_BOOK,
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
    }
    pub(crate) async fn delete_book(
        &mut self,
        book_id: i32,
    ) -> Result<(), sqlc_core::Error> {
        self.client.execute(DELETE_BOOK, &[&book_id]).await?;
        Ok(())
    }
    pub(crate) async fn get_author(
        &mut self,
        author_id: i32,
    ) -> Result<Author, sqlc_core::Error> {
        let row = self.client.query_one(GET_AUTHOR, &[&author_id]).await?;
        Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
    }
    pub(crate) async fn get_book(
        &mut self,
        book_id: i32,
    ) -> Result<Book, sqlc_core::Error> {
        let row = self.client.query_one(GET_BOOK, &[&book_id]).await?;
        Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
    }
    pub(crate) async fn say_hello(
        &mut self,
        s: String,
    ) -> Result<String, sqlc_core::Error> {
        let row = self.client.query_one(SAY_HELLO, &[&s]).await?;
        Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
    }
    pub(crate) async fn update_book(
        &mut self,
        arg: UpdateBookParams,
    ) -> Result<(), sqlc_core::Error> {
        self.client.execute(UPDATE_BOOK, &[&arg.title, &arg.tags, &arg.book_id]).await?;
        Ok(())
    }
    pub(crate) async fn update_book_isbn(
        &mut self,
        arg: UpdateBookIsbnParams,
    ) -> Result<(), sqlc_core::Error> {
        self.client
            .execute(UPDATE_BOOK_ISBN, &[&arg.title, &arg.tags, &arg.book_id, &arg.isbn])
            .await?;
        Ok(())
    }
}
