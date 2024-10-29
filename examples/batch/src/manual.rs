use futures::TryStreamExt;

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
}
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
pub(crate) struct Author {
    pub author_id: i32,
    pub name: String,
    pub biography: Option<serde_json::Value>,
}
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
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
#[derive(Clone, Debug, sqlc_derive::FromPostgresRow, PartialEq)]
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

async fn create_book(
    pool: deadpool_postgres::Pool,
    stmt: tokio_postgres::Statement,
    arg: CreateBookParams,
) -> Result<Book, sqlc_core::Error> {
    let client = pool.clone().get().await?;
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

    pub(crate) async fn create_author(&mut self, name: String) -> Result<Author, sqlc_core::Error> {
        let row = self
            .client()
            .await
            .query_one(CREATE_AUTHOR, &[&name])
            .await?;
        Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
    }
    pub(crate) async fn create_book(
        &mut self,
        arg: Vec<CreateBookParams>,
    ) -> Result<impl futures::Stream<Item = Result<Book, sqlc_core::Error>>, sqlc_core::Error> {
        let stmt = self.client().await.prepare(CREATE_BOOK).await?;
        Ok(sqlc_core::BatchResults::new(
            self.pool.clone(),
            arg,
            stmt,
            create_book,
        ))
    }
}

pub(crate) async fn execute(pool: deadpool_postgres::Pool) {
    let mut queries = Queries::new(pool.clone());

    let a = queries
        .create_author("Unknown Master".to_string())
        .await
        .unwrap();

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
            book_type: BookType::Fiction,
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

    let new_books = queries
        .create_book(new_book_params.clone())
        .await
        .expect("failed to create batch results")
        .try_collect::<Vec<_>>()
        .await
        .expect("failed to collect batch results 1");
    println!("books: {:?}", new_books);
    assert_eq!(new_books.len(), new_book_params.len());
}
