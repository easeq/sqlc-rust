use super::*;
use futures::stream::StreamExt;
use std::borrow::Borrow;
use testcontainers::{clients::Cli, images::postgres::Postgres};
use tokio_postgres::NoTls;

async fn setup_db() -> tokio_postgres::Client {
    let docker = Cli::default();
    let postgres = docker.run(Postgres::default());
    let port = postgres.get_host_port_ipv4(5432);

    let (client, connection) = tokio_postgres::connect(
        &format!(
            "host=localhost port={} user=postgres password=postgres dbname=postgres",
            port
        ),
        NoTls,
    )
    .await
    .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client
        .batch_execute("CREATE TEMP TABLE test (id SERIAL PRIMARY KEY, name VARCHAR)")
        .await
        .unwrap();
    client
}

#[tokio::test]
async fn test_prepare_transaction() {
    let client = setup_db().await;
    let transaction = client.transaction().await.unwrap();
    let stmt = transaction.prepare("SELECT * FROM test").await.unwrap();
    assert_eq!(stmt.to_string(), "SELECT * FROM test");
}

#[tokio::test]
async fn test_prepare_client() {
    let client = setup_db().await;
    let stmt = client.prepare("SELECT * FROM test").await.unwrap();
    assert_eq!(stmt.to_string(), "SELECT * FROM test");
}

#[tokio::test]
async fn test_execute_transaction() {
    let client = setup_db().await;
    let transaction = client.transaction().await.unwrap();
    let stmt = transaction
        .prepare("INSERT INTO test (name) VALUES ($1)")
        .await
        .unwrap();
    let rows_affected = transaction.execute(&stmt, &[&"test"]).await.unwrap();
    assert_eq!(rows_affected, 1);
}

#[tokio::test]
async fn test_execute_client() {
    let client = setup_db().await;
    let stmt = client
        .prepare("INSERT INTO test (name) VALUES ($1)")
        .await
        .unwrap();
    let rows_affected = client.execute(&stmt, &[&"test"]).await.unwrap();
    assert_eq!(rows_affected, 1);
}

#[tokio::test]
async fn test_query_one_transaction() {
    let client = setup_db().await;
    let transaction = client.transaction().await.unwrap();
    transaction
        .execute("INSERT INTO test (name) VALUES ('test1')", &[])
        .await
        .unwrap();
    let stmt = transaction
        .prepare("SELECT * FROM test WHERE name = $1")
        .await
        .unwrap();
    let row = transaction.query_one(&stmt, &[&"test1"]).await.unwrap();
    let name: &str = row.get("name");
    assert_eq!(name, "test1");
}

#[tokio::test]
async fn test_query_one_client() {
    let client = setup_db().await;
    client
        .execute("INSERT INTO test (name) VALUES ('test1')", &[])
        .await
        .unwrap();
    let stmt = client
        .prepare("SELECT * FROM test WHERE name = $1")
        .await
        .unwrap();
    let row = client.query_one(&stmt, &[&"test1"]).await.unwrap();
    let name: &str = row.get("name");
    assert_eq!(name, "test1");
}

#[tokio::test]
async fn test_query_transaction() {
    let client = setup_db().await;
    let transaction = client.transaction().await.unwrap();
    transaction
        .execute("INSERT INTO test (name) VALUES ('test1'), ('test2')", &[])
        .await
        .unwrap();
    let stmt = transaction.prepare("SELECT * FROM test").await.unwrap();
    let rows = transaction.query(&stmt, &[]).await.unwrap();
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn test_query_client() {
    let client = setup_db().await;
    client
        .execute("INSERT INTO test (name) VALUES ('test1'), ('test2')", &[])
        .await
        .unwrap();
    let stmt = client.prepare("SELECT * FROM test").await.unwrap();
    let rows = client.query(&stmt, &[]).await.unwrap();
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn test_batch_execute_transaction() {
    let client = setup_db().await;
    let transaction = client.transaction().await.unwrap();
    let arg_list = vec![vec![&"test1"], vec![&"test2"]];
    let stream = transaction
        .batch_execute("INSERT INTO test (name) VALUES ($1)", arg_list.iter())
        .await
        .unwrap();
    stream
        .for_each(|res| async {
            res.unwrap();
        })
        .await;
    let rows = transaction.query("SELECT * FROM test", &[]).await.unwrap();
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn test_batch_execute_client() {
    let client = setup_db().await;
    let arg_list = vec![vec![&"test1"], vec![&"test2"]];
    let stream = client
        .batch_execute("INSERT INTO test (name) VALUES ($1)", arg_list.iter())
        .await
        .unwrap();
    stream
        .for_each(|res| async {
            res.unwrap();
        })
        .await;
    let rows = client.query("SELECT * FROM test", &[]).await.unwrap();
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn test_batch_one_transaction() {
    let client = setup_db().await;
    let transaction = client.transaction().await.unwrap();
    transaction
        .execute("INSERT INTO test (name) VALUES ('test1'), ('test2')", &[])
        .await
        .unwrap();
    let arg_list = vec![vec![&"test1"], vec![&"test2"]];
    let stream = transaction
        .batch_one("SELECT * FROM test WHERE name = $1", arg_list.iter())
        .await
        .unwrap();
    let mut count = 0;
    stream
        .for_each(|res| async {
            let row = res.unwrap();
            assert!(row.get::<_, &str>("name") == "test1" || row.get::<_, &str>("name") == "test2");
            count += 1;
        })
        .await;
    assert_eq!(count, 2);
}

#[tokio::test]
async fn test_batch_one_client() {
    let client = setup_db().await;
    client
        .execute("INSERT INTO test (name) VALUES ('test1'), ('test2')", &[])
        .await
        .unwrap();
    let arg_list = vec![vec![&"test1"], vec![&"test2"]];
    let stream = client
        .batch_one("SELECT * FROM test WHERE name = $1", arg_list.iter())
        .await
        .unwrap();
    let mut count = 0;
    stream
        .for_each(|res| async {
            let row = res.unwrap();
            assert!(row.get::<_, &str>("name") == "test1" || row.get::<_, &str>("name") == "test2");
            count += 1;
        })
        .await;
    assert_eq!(count, 2);
}

#[tokio::test]
async fn test_batch_many_transaction() {
    let client = setup_db().await;
    let transaction = client.transaction().await.unwrap();
    transaction
        .execute("INSERT INTO test (name) VALUES ('test1'), ('test2')", &[])
        .await
        .unwrap();
    let arg_list = vec![vec![&"test1"], vec![&"test2"]];
    let stream = transaction
        .batch_many("SELECT * FROM test WHERE name = $1", arg_list.iter())
        .await
        .unwrap();
    let mut count = 0;
    stream
        .for_each(|res| async {
            let rows = res.unwrap().collect::<Vec<_>>().await;
            assert_eq!(rows.len(), 1);
            count += 1;
        })
        .await;
    assert_eq!(count, 2);
}

#[tokio::test]
async fn test_batch_many_client() {
    let client = setup_db().await;
    client
        .execute("INSERT INTO test (name) VALUES ('test1'), ('test2')", &[])
        .await
        .unwrap();
    let arg_list = vec![vec![&"test1"], vec![&"test2"]];
    let stream = client
        .batch_many("SELECT * FROM test WHERE name = $1", arg_list.iter())
        .await
        .unwrap();
    let mut count = 0;
    stream
        .for_each(|res| async {
            let rows = res.unwrap().collect::<Vec<_>>().await;
            assert_eq!(rows.len(), 1);
            count += 1;
        })
        .await;
    assert_eq!(count, 2);
}
