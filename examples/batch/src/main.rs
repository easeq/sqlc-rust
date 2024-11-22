use deadpool_postgres::{Config, Runtime};
use postgresql_embedded::{PostgreSQL, Result};
use std::ops::DerefMut;
use tokio_postgres::NoTls;

// #[path = "./db/gen.rs"]
// pub mod db;

// pub mod generated;
pub mod manual;

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

    manual::execute(pool.clone()).await;
    // generated::execute(pool.clone()).await;

    postgresql.stop().await
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_all_queries() {
        crate::main().unwrap()
    }
}
