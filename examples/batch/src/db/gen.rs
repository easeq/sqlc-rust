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
    pub biography: Option<serde_json::Value>,
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
pub(crate) struct UpdateBookParams {
    pub title: String,
    pub tags: Vec<String>,
    pub book_id: i32,
}
#[sqlc_derive::batch_result_type]
pub(crate) struct DeleteBookBatchResults {
    #[batch_param]
    param: i32,
    #[batch_result]
    result: (),
}
#[sqlc_derive::batch_result_type]
pub(crate) struct DeleteBookNamedFuncBatchResults {
    #[batch_param]
    param: i32,
    #[batch_result]
    result: (),
}
#[sqlc_derive::batch_result_type]
pub(crate) struct DeleteBookNamedSignBatchResults {
    #[batch_param]
    param: i32,
    #[batch_result]
    result: (),
}
#[sqlc_derive::batch_result_type]
pub(crate) struct BooksByYearBatchResults {
    #[batch_param]
    param: i32,
    #[batch_result]
    result: std::pin::Pin<Box<futures::stream::Iter<std::vec::IntoIter<Book>>>>,
}
#[sqlc_derive::batch_result_type]
pub(crate) struct CreateBookBatchResults {
    #[batch_param]
    param: CreateBookParams,
    #[batch_result]
    result: Book,
}
#[sqlc_derive::batch_result_type]
pub(crate) struct UpdateBookBatchResults {
    #[batch_param]
    param: UpdateBookParams,
    #[batch_result]
    result: (),
}
#[sqlc_derive::batch_result_type]
pub(crate) struct GetBiographyBatchResults {
    #[batch_param]
    param: i32,
    #[batch_result]
    result: serde_json::Value,
}
#[derive(Clone)]
pub struct Queries {
    pool: deadpool_postgres::Pool,
}
impl Queries {
    pub fn new(pool: deadpool_postgres::Pool) -> Self {
        Self { pool }
    }
    pub async fn client(&self) -> deadpool::managed::Object<deadpool_postgres::Manager> {
        let client = self.pool.get().await.unwrap();
        client
    }
    pub(crate) async fn books_by_year(
        &mut self,
        year: Vec<i32>,
    ) -> Result<BooksByYearBatchResults, sqlc_core::Error> {
        let fut: BooksByYearBatchResultsFn = Box::new(|
            pool: deadpool_postgres::Pool,
            stmt: tokio_postgres::Statement,
            year: i32|
        {
            Box::pin(async move {
                let client = pool.clone().get().await.ok()?;
                let rows = client.query(&stmt, &[&year]).await.ok()?;
                let mut result: Vec<Book> = vec![];
                for row in rows {
                    result.push(sqlc_core::FromPostgresRow::from_row(&row).ok()?);
                }
                Some(Box::pin(futures::stream::iter(result)))
            })
        });
        let stmt = self.client().await.prepare(BOOKS_BY_YEAR).await?;
        Ok(BooksByYearBatchResults::new(self.pool.clone(), year, stmt, fut))
    }
    pub(crate) async fn create_author(
        &mut self,
        name: String,
    ) -> Result<Author, sqlc_core::Error> {
        let row = self.client().await.query_one(CREATE_AUTHOR, &[&name]).await?;
        Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
    }
    pub(crate) async fn create_book(
        &mut self,
        arg: Vec<CreateBookParams>,
    ) -> Result<CreateBookBatchResults, sqlc_core::Error> {
        let fut: CreateBookBatchResultsFn = Box::new(|
            pool: deadpool_postgres::Pool,
            stmt: tokio_postgres::Statement,
            arg: CreateBookParams|
        {
            Box::pin(async move {
                let client = pool.clone().get().await.ok()?;
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
                    .await
                    .ok()?;
                Some(sqlc_core::FromPostgresRow::from_row(&row).ok()?)
            })
        });
        let stmt = self.client().await.prepare(CREATE_BOOK).await?;
        Ok(CreateBookBatchResults::new(self.pool.clone(), arg, stmt, fut))
    }
    pub(crate) async fn delete_book(
        &mut self,
        book_id: Vec<i32>,
    ) -> Result<DeleteBookBatchResults, sqlc_core::Error> {
        let fut: DeleteBookBatchResultsFn = Box::new(|
            pool: deadpool_postgres::Pool,
            stmt: tokio_postgres::Statement,
            book_id: i32|
        {
            Box::pin(async move {
                let client = pool.clone().get().await.ok()?;
                client.execute(&stmt, &[&book_id]).await.ok()?;
                Some(())
            })
        });
        let stmt = self.client().await.prepare(DELETE_BOOK).await?;
        Ok(DeleteBookBatchResults::new(self.pool.clone(), book_id, stmt, fut))
    }
    pub(crate) async fn delete_book_exec_result(
        &mut self,
        book_id: i32,
    ) -> Result<(), sqlc_core::Error> {
        self.client().await.execute(DELETE_BOOK_EXEC_RESULT, &[&book_id]).await?;
        Ok(())
    }
    pub(crate) async fn delete_book_named_func(
        &mut self,
        book_id: Vec<i32>,
    ) -> Result<DeleteBookNamedFuncBatchResults, sqlc_core::Error> {
        let fut: DeleteBookNamedFuncBatchResultsFn = Box::new(|
            pool: deadpool_postgres::Pool,
            stmt: tokio_postgres::Statement,
            book_id: i32|
        {
            Box::pin(async move {
                let client = pool.clone().get().await.ok()?;
                client.execute(&stmt, &[&book_id]).await.ok()?;
                Some(())
            })
        });
        let stmt = self.client().await.prepare(DELETE_BOOK_NAMED_FUNC).await?;
        Ok(DeleteBookNamedFuncBatchResults::new(self.pool.clone(), book_id, stmt, fut))
    }
    pub(crate) async fn delete_book_named_sign(
        &mut self,
        book_id: Vec<i32>,
    ) -> Result<DeleteBookNamedSignBatchResults, sqlc_core::Error> {
        let fut: DeleteBookNamedSignBatchResultsFn = Box::new(|
            pool: deadpool_postgres::Pool,
            stmt: tokio_postgres::Statement,
            book_id: i32|
        {
            Box::pin(async move {
                let client = pool.clone().get().await.ok()?;
                client.execute(&stmt, &[&book_id]).await.ok()?;
                Some(())
            })
        });
        let stmt = self.client().await.prepare(DELETE_BOOK_NAMED_SIGN).await?;
        Ok(DeleteBookNamedSignBatchResults::new(self.pool.clone(), book_id, stmt, fut))
    }
    pub(crate) async fn get_author(
        &mut self,
        author_id: i32,
    ) -> Result<Author, sqlc_core::Error> {
        let row = self.client().await.query_one(GET_AUTHOR, &[&author_id]).await?;
        Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
    }
    pub(crate) async fn get_biography(
        &mut self,
        author_id: Vec<i32>,
    ) -> Result<GetBiographyBatchResults, sqlc_core::Error> {
        let fut: GetBiographyBatchResultsFn = Box::new(|
            pool: deadpool_postgres::Pool,
            stmt: tokio_postgres::Statement,
            author_id: i32|
        {
            Box::pin(async move {
                let client = pool.clone().get().await.ok()?;
                let row = client.query_one(&stmt, &[&author_id]).await.ok()?;
                Some(sqlc_core::FromPostgresRow::from_row(&row).ok()?)
            })
        });
        let stmt = self.client().await.prepare(GET_BIOGRAPHY).await?;
        Ok(GetBiographyBatchResults::new(self.pool.clone(), author_id, stmt, fut))
    }
    pub(crate) async fn update_book(
        &mut self,
        arg: Vec<UpdateBookParams>,
    ) -> Result<UpdateBookBatchResults, sqlc_core::Error> {
        let fut: UpdateBookBatchResultsFn = Box::new(|
            pool: deadpool_postgres::Pool,
            stmt: tokio_postgres::Statement,
            arg: UpdateBookParams|
        {
            Box::pin(async move {
                let client = pool.clone().get().await.ok()?;
                client
                    .execute(&stmt, &[&arg.title, &arg.tags, &arg.book_id])
                    .await
                    .ok()?;
                Some(())
            })
        });
        let stmt = self.client().await.prepare(UPDATE_BOOK).await?;
        Ok(UpdateBookBatchResults::new(self.pool.clone(), arg, stmt, fut))
    }
}
