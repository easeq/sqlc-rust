use deadpool_postgres::{Config, Runtime};
use futures::StreamExt;
use futures::TryStreamExt;
use postgresql_embedded::{PostgreSQL, Result};
use std::ops::DerefMut;
use tokio_postgres::NoTls;

#[path = "./db/gen.rs"]
pub mod db;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("postgresql/migrations");
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut postgresql = PostgreSQL::default();
    postgresql.setup().await?;
    postgresql.start().await?;

    let database_name = "test";
    postgresql.create_database(database_name).await?;

    let settings = postgresql.settings();
    let mut config = Config::new();
    config.host = Some(settings.host.clone());
    config.port = Some(settings.port.clone());
    config.user = Some(settings.username.clone());
    config.password = Some(settings.password.clone());
    config.dbname = Some(database_name.to_string());

    let pool = config
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("failed to create pool");
    let mut db_client = pool.get().await.expect("failed to get client from pool");

    let client = db_client.deref_mut().deref_mut();

    client
        .execute("CREATE extension hstore", &[])
        .await
        .unwrap();

    embedded::migrations::runner()
        .run_async(client)
        .await
        .expect("failed to load migrations");

    let mut queries = db::Queries::new(pool.clone());

    let a = queries
        .create_author("Unknown Master".to_string())
        .await
        .unwrap();

    let new_book_params = vec![
        db::CreateBookParams {
            author_id: a.author_id,
            isbn: "1".to_string(),
            title: "my book title".to_string(),
            book_type: db::BookType::Fiction,
            year: 2016,
            available: time::OffsetDateTime::now_utc(),
            tags: vec![],
        },
        db::CreateBookParams {
            author_id: a.author_id,
            isbn: "2".to_string(),
            title: "the second book".to_string(),
            book_type: db::BookType::Fiction,
            year: 2016,
            available: time::OffsetDateTime::now_utc(),
            tags: vec!["cool".to_string(), "unique".to_string()],
        },
        db::CreateBookParams {
            author_id: a.author_id,
            isbn: "3".to_string(),
            title: "the third book".to_string(),
            book_type: db::BookType::Fiction,
            year: 2001,
            available: time::OffsetDateTime::now_utc(),
            tags: vec!["cool".to_string()],
        },
        db::CreateBookParams {
            author_id: a.author_id,
            isbn: "4".to_string(),
            title: "4th place finisher".to_string(),
            book_type: db::BookType::Fiction,
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

    // let new_books = queries
    //     .create_book(new_book_params.clone())
    //     .await
    //     .expect("failed to create batch results")
    //     .try_collect::<Vec<_>>()
    //     .await
    //     .expect("failed to collect batch results 2");
    // println!("books: {:?}", new_books);
    // assert_eq!(new_books.len(), new_book_params.len());

    let update_books_params = vec![db::UpdateBookParams {
        book_id: new_books[1].book_id,
        title: "changed second title".to_string(),
        tags: vec!["cool".to_string(), "disastor".to_string()],
    }];

    queries
        .update_book(update_books_params.clone())
        .await
        .expect("failed to create update books results")
        .try_collect::<Vec<_>>()
        .await
        .expect("failed to update books");

    let select_books_by_title_year_params = vec![2001, 2016];
    let books: Vec<(db::Book, db::Author)> = queries
        .books_by_year(select_books_by_title_year_params.clone())
        .await
        .expect("failed to fetch books by year")
        .try_flatten()
        .then(|book| {
            let queries = queries.clone();
            async move {
                let book = book?;
                println!(
                    "Book {book_id} ({book_type:?}): {book_title} available: {book_available}",
                    book_id = book.book_id,
                    book_type = book.book_type,
                    book_title = book.title,
                    book_available = book.available,
                );

                let author = queries.clone().get_author(book.author_id).await.unwrap();
                Ok::<(db::Book, db::Author), sqlc_core::Error>((book, author))
            }
        })
        .try_collect()
        .await
        .expect("failed to fetch books by year");

    println!("{:?}", books);

    let delete_books_params = new_books
        .iter()
        .map(|new_book| new_book.book_id)
        .collect::<Vec<_>>();

    let want_num_deletes_processed = 2;
    let deleted_books = queries
        .delete_book(delete_books_params)
        .await
        .expect("failed to delete books")
        .take(want_num_deletes_processed)
        .collect::<Vec<_>>()
        .await;

    assert_eq!(deleted_books.len(), want_num_deletes_processed);

    postgresql.stop().await
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_all_queries() {
        crate::main().unwrap()
    }
}
