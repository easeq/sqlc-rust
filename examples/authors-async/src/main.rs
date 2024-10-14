use postgresql_embedded::{PostgreSQL, Result};

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
    let conn_str = format!(
        "postgresql://{username}:{password}@{host}:{port}/{database_name}",
        host = settings.host,
        port = settings.port,
        username = settings.username,
        password = settings.password,
        database_name = database_name,
    );
    let (mut db_client, connection) = tokio_postgres::connect(&conn_str, tokio_postgres::NoTls)
        .await
        .expect("failed to connect to db");

    tokio::task::spawn(async move {
        let _ = connection.await;
    });

    embedded::migrations::runner()
        .run_async(&mut db_client)
        .await
        .expect("failed to load migrations");

    let mut queries = db::Queries::new(db_client);

    let authors = queries.list_authors().await.unwrap();
    assert_eq!(authors.len(), 0);

    let author_res_err = queries.get_author(1).await.is_err();
    assert_eq!(author_res_err, true);

    let delete_res = queries.delete_author(1).await.is_ok();
    assert_eq!(delete_res, true);

    let author1_req = db::CreateAuthorParams {
        name: "Author 1".to_string(),
        bio: None,
    };
    let author1_res = queries.create_author(author1_req.clone()).await.unwrap();
    assert_eq!(author1_res.name, author1_req.name);
    assert_eq!(author1_res.bio, author1_req.bio.clone());
    assert!(author1_res.id > 0);

    let mut authors_list_prepared = vec![author1_res.clone()];
    let authors = queries.list_authors().await.unwrap();
    assert_eq!(authors.len(), 1);
    assert_eq!(authors, authors_list_prepared);

    let author2_req = db::CreateAuthorParams {
        name: "Author 2".to_string(),
        bio: Some("My name is Author 2".to_string()),
    };
    let author2_res = queries.create_author(author2_req.clone()).await.unwrap();
    assert_eq!(author2_res.name, author2_req.name);
    assert_eq!(author2_res.bio, author2_req.bio);
    assert!(author2_res.id > 1);

    authors_list_prepared.push(author2_res.clone());

    let authors = queries.list_authors().await.unwrap();
    assert_eq!(authors.len(), 2);
    assert_eq!(authors, authors_list_prepared);

    let author = queries.get_author(1).await.unwrap();
    assert_eq!(author, author1_res);

    queries.delete_author(1).await.unwrap();
    let authors = queries.list_authors().await.unwrap();
    assert_eq!(authors.len(), 1);
    assert_eq!(authors, authors_list_prepared[1..]);

    postgresql.stop().await
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_all_queries() {
        crate::main().unwrap()
    }
}
