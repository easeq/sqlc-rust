use postgresql_embedded::blocking::PostgreSQL;
use postgresql_embedded::Result;

#[path = "./db/gen.rs"]
pub mod db;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("postgresql/migrations");
}

fn main() -> Result<()> {
    let mut postgresql = PostgreSQL::default();
    postgresql.setup()?;
    postgresql.start()?;

    let database_name = "test";
    postgresql.create_database(database_name)?;

    let settings = postgresql.settings();
    let mut client = postgres::Client::connect(
        format!(
            "postgresql://{username}:{password}@{host}:{port}/{database_name}",
            host = settings.host,
            port = settings.port,
            username = settings.username,
            password = settings.password,
            database_name = database_name,
        )
        .as_str(),
        postgres::NoTls,
    )
    .unwrap();

    embedded::migrations::runner().run(&mut client).unwrap();

    let mut queries = db::Queries::new(client);

    let authors = queries.list_authors().unwrap();
    assert_eq!(authors.len(), 0);

    let author_res_err = queries.get_author(1).is_err();
    assert_eq!(author_res_err, true);

    let delete_res = queries.delete_author(1).is_ok();
    assert_eq!(delete_res, true);

    let author1_req = db::CreateAuthorParams {
        name: "Author 1".to_string(),
        bio: None,
    };
    let author1_res = queries.create_author(author1_req.clone()).unwrap();
    assert_eq!(author1_res.name, author1_req.name);
    assert_eq!(author1_res.bio, author1_req.bio.clone());
    assert!(author1_res.id > 0);

    let mut authors_list_prepared = vec![author1_res.clone()];
    let authors = queries.list_authors().unwrap();
    assert_eq!(authors.len(), 1);
    assert_eq!(authors, authors_list_prepared);

    let author2_req = db::CreateAuthorParams {
        name: "Author 2".to_string(),
        bio: Some("My name is Author 2".to_string()),
    };
    let author2_res = queries.create_author(author2_req.clone()).unwrap();
    assert_eq!(author2_res.name, author2_req.name);
    assert_eq!(author2_res.bio, author2_req.bio);
    assert!(author2_res.id > 1);

    authors_list_prepared.push(author2_res.clone());

    let authors = queries.list_authors().unwrap();
    assert_eq!(authors.len(), 2);
    assert_eq!(authors, authors_list_prepared);

    let author = queries.get_author(1).unwrap();
    assert_eq!(author, author1_res);

    queries.delete_author(1).unwrap();
    let authors = queries.list_authors().unwrap();
    assert_eq!(authors.len(), 1);
    assert_eq!(authors, authors_list_prepared[1..]);

    postgresql.stop()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_all_queries() {
        crate::main().unwrap()
    }
}
