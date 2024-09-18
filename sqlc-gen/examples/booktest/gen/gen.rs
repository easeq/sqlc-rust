#[derive(Debug, Display)]
pub enum BookType {
    Fiction,
    Nonfiction,
}
const GET_AUTHOR: &str = r#"
select author_id, name
from authors
where author_id = $1
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct GetAuthorParams {
    pub(crate) author_id: u16,
}
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct GetAuthorRow {
    pub(crate) author_id: u16,
    pub(crate) name: String,
}
const GET_BOOK: &str = r#"
select book_id, author_id, isbn, book_type, title, year, available, tags
from books
where book_id = $1
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct GetBookParams {
    pub(crate) book_id: u16,
}
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct GetBookRow {
    pub(crate) book_id: u16,
    pub(crate) author_id: i32,
    pub(crate) isbn: String,
    pub(crate) book_type: String,
    pub(crate) title: String,
    pub(crate) year: i32,
    pub(crate) available: String,
    pub(crate) tags: Vec<String>,
}
const DELETE_BOOK: &str = r#"
delete from books
where book_id = $1
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct DeleteBookParams {
    pub(crate) book_id: u16,
}
const BOOKS_BY_TITLE_YEAR: &str = r#"
select book_id, author_id, isbn, book_type, title, year, available, tags
from books
where title = $1 and year = $2
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct BooksByTitleYearParams {
    pub(crate) title: String,
    pub(crate) year: i32,
}
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct BooksByTitleYearRow {
    pub(crate) book_id: u16,
    pub(crate) author_id: i32,
    pub(crate) isbn: String,
    pub(crate) book_type: String,
    pub(crate) title: String,
    pub(crate) year: i32,
    pub(crate) available: String,
    pub(crate) tags: Vec<String>,
}
const BOOKS_BY_TAGS: &str = r#"
select book_id, title, name, isbn, tags
from books
left join authors on books.author_id = authors.author_id
where tags && $1::varchar[]
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct BooksByTagsParams {
    pub(crate) _1: Vec<String>,
}
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct BooksByTagsRow {
    pub(crate) book_id: u16,
    pub(crate) title: String,
    pub(crate) name: Option<String>,
    pub(crate) isbn: String,
    pub(crate) tags: Vec<String>,
}
const CREATE_AUTHOR: &str = r#"
INSERT INTO authors (name) VALUES ($1)
RETURNING author_id, name
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct CreateAuthorParams {
    pub(crate) name: String,
}
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct CreateAuthorRow {
    pub(crate) author_id: u16,
    pub(crate) name: String,
}
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
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct CreateBookParams {
    pub(crate) author_id: i32,
    pub(crate) isbn: String,
    pub(crate) book_type: String,
    pub(crate) title: String,
    pub(crate) year: i32,
    pub(crate) available: String,
    pub(crate) tags: Vec<String>,
}
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct CreateBookRow {
    pub(crate) book_id: u16,
    pub(crate) author_id: i32,
    pub(crate) isbn: String,
    pub(crate) book_type: String,
    pub(crate) title: String,
    pub(crate) year: i32,
    pub(crate) available: String,
    pub(crate) tags: Vec<String>,
}
const UPDATE_BOOK: &str = r#"
UPDATE books
SET title = $1, tags = $2
WHERE book_id = $3
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct UpdateBookParams {
    pub(crate) title: String,
    pub(crate) tags: Vec<String>,
    pub(crate) book_id: u16,
}
const UPDATE_BOOK_ISBN: &str = r#"
UPDATE books
SET title = $1, tags = $2, isbn = $4
WHERE book_id = $3
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct UpdateBookIsbnParams {
    pub(crate) title: String,
    pub(crate) tags: Vec<String>,
    pub(crate) book_id: u16,
    pub(crate) isbn: String,
}
const SAY_HELLO: &str = r#"
select say_hello
from say_hello($1)
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct SayHelloParams {
    pub(crate) s: String,
}
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct SayHelloRow {
    pub(crate) say_hello: Option<String>,
}
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
    pub fn get_author(&self, params: GetAuthorParams) -> anyhow::Result<GetAuthorRow> {
        let row: GetAuthorRow = self.client.query_one(GET_AUTHOR, &[&params.author_id])?;
        Ok(row)
    }
    pub fn get_book(&self, params: GetBookParams) -> anyhow::Result<GetBookRow> {
        let row: GetBookRow = self.client.query_one(GET_BOOK, &[&params.book_id])?;
        Ok(row)
    }
    pub fn delete_book(&self, params: DeleteBookParams) -> anyhow::Result<()> {
        self.client.execute(DELETE_BOOK, &[&params.book_id])?;
        Ok(())
    }
    pub fn books_by_title_year(
        &self,
        params: BooksByTitleYearParams,
    ) -> anyhow::Result<Vec<BooksByTitleYearRow>> {
        let rows = self
            .client
            .query(BOOKS_BY_TITLE_YEAR, &[&params.title, &params.year])?;
        let result: Vec<BooksByTitleYearRow> = vec![];
        for row in rows {
            result.push(row.into());
        }
        Ok(result)
    }
    pub fn books_by_tags(
        &self,
        params: BooksByTagsParams,
    ) -> anyhow::Result<Vec<BooksByTagsRow>> {
        let rows = self.client.query(BOOKS_BY_TAGS, &[&params._1])?;
        let result: Vec<BooksByTagsRow> = vec![];
        for row in rows {
            result.push(row.into());
        }
        Ok(result)
    }
    pub fn create_author(
        &self,
        params: CreateAuthorParams,
    ) -> anyhow::Result<CreateAuthorRow> {
        let row: CreateAuthorRow = self
            .client
            .query_one(CREATE_AUTHOR, &[&params.name])?;
        Ok(row)
    }
    pub fn create_book(
        &self,
        params: CreateBookParams,
    ) -> anyhow::Result<CreateBookRow> {
        let row: CreateBookRow = self
            .client
            .query_one(
                CREATE_BOOK,
                &[
                    &params.author_id,
                    &params.isbn,
                    &params.book_type,
                    &params.title,
                    &params.year,
                    &params.available,
                    &params.tags,
                ],
            )?;
        Ok(row)
    }
    pub fn update_book(&self, params: UpdateBookParams) -> anyhow::Result<()> {
        self.client
            .execute(UPDATE_BOOK, &[&params.title, &params.tags, &params.book_id])?;
        Ok(())
    }
    pub fn update_book_isbn(&self, params: UpdateBookIsbnParams) -> anyhow::Result<()> {
        self.client
            .execute(
                UPDATE_BOOK_ISBN,
                &[&params.title, &params.tags, &params.book_id, &params.isbn],
            )?;
        Ok(())
    }
    pub fn say_hello(&self, params: SayHelloParams) -> anyhow::Result<SayHelloRow> {
        let row: SayHelloRow = self.client.query_one(SAY_HELLO, &[&params.s])?;
        Ok(row)
    }
}
