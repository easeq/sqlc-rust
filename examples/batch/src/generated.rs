use crate::db;
use futures::StreamExt;
use futures::TryStreamExt;
use itertools::Itertools;
use std::ops::{Deref, DerefMut};

pub async fn execute(pool: deadpool_postgres::Pool) {
    let mut db_client = pool.get().await.expect("failed to get client from pool");
    let client = db_client.deref_mut().deref_mut();

    let a = db::create_author(client, "Unknown Master".to_string())
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
    let new_books = db::create_book(client, &new_book_params)
        .await
        .expect("failed to create batch results")
        .buffered(10)
        .try_collect::<Vec<_>>()
        .await
        .expect("failed to collect batch results 1");
    println!("books: {:#?}", new_books);
    assert_eq!(new_books.len(), new_book_params.len());

    let new_books_2 = db::create_book(client, &new_book_params)
        .await
        .expect("failed to create batch results")
        .buffer_unordered(2)
        .try_collect::<Vec<_>>()
        .await;
    println!("new books 2 err: {:#?}", new_books_2);
    assert_eq!(new_books_2.is_err(), true);

    let update_books_params = vec![db::UpdateBookParams {
        book_id: new_books[1].book_id,
        title: "changed second title".to_string(),
        tags: vec!["cool".to_string(), "disastor".to_string()],
    }];

    db::update_book(client, &update_books_params)
        .await
        .expect("failed to create update books results")
        .buffer_unordered(1)
        .try_collect::<Vec<_>>()
        .await
        .expect("failed to update books");

    let select_books_by_title_year_params = vec![2001, 2016];
    let books: Vec<(db::Book, db::Author)> =
        db::books_by_year(client, &select_books_by_title_year_params)
            .await
            .expect("failed to fetch books by year")
            .buffer_unordered(3)
            .try_flatten()
            .then(|book| {
                let pool = pool.clone();

                async move {
                    let db_client = pool.get().await.expect("failed to get client from pool");
                    let client = db_client.deref().deref();

                    let book = book?.unwrap();
                    println!(
                        "Book {book_id} ({book_type:?}): {book_title} available: {book_available}",
                        book_id = book.book_id,
                        book_type = book.book_type,
                        book_title = book.title,
                        book_available = book.available,
                    );

                    let author = db::get_author(client, book.author_id).await.unwrap();
                    Ok::<(db::Book, db::Author), sqlc_core::Error>((book, author))
                }
            })
            .try_collect()
            .await
            .expect("failed to fetch books by year");

    println!("{books:#?}");

    let delete_books_params = new_books
        .iter()
        .map(|new_book| new_book.book_id)
        .collect::<Vec<_>>();

    let want_num_deletes_processed = 2;
    let deleted_books = db::delete_book(client, &delete_books_params)
        .await
        .expect("failed to delete books")
        .take(want_num_deletes_processed)
        .buffer_unordered(10)
        .collect::<Vec<_>>()
        .await;

    assert_eq!(deleted_books.len(), want_num_deletes_processed);

    let transaction = client
        .transaction()
        .await
        .expect("could not create transaction");

    let update_books_params = vec![
        db::UpdateBookParams {
            book_id: new_books[3].book_id,
            title: "changed 4th txn title".to_string(),
            tags: vec!["cool".to_string(), "disastor".to_string()],
        },
        db::UpdateBookParams {
            book_id: new_books[2].book_id,
            title: "changed third txn title".to_string(),
            tags: vec!["cool".to_string(), "disastor".to_string()],
        },
    ];

    db::update_book(&transaction, &update_books_params)
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

    let books: Vec<_> = db::all_books(client)
        .await
        .expect("failed to fetch all books")
        .try_collect()
        .expect("failed to collect all books");
    println!("{books:#?}");
}
