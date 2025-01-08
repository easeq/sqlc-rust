use crate::{FromPostgresRow, Result};
use async_trait::async_trait;
use tokio_postgres::types::ToSql;
use tokio_postgres::{Client, Statement, ToStatement, Transaction};

#[async_trait]
pub trait DBTX: Send + Sync {
    async fn prepare(&self, query: &str) -> Result<Statement>;
    async fn execute<T>(&self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<u64>
    where
        T: ?Sized + ToStatement + Sync + Send;
    async fn query_one<T, R>(&self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<R>
    where
        T: ?Sized + ToStatement + Sync + Send,
        R: FromPostgresRow;
    async fn query<T, R>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Box<dyn std::iter::Iterator<Item = crate::Result<R>>>>
    where
        T: ?Sized + ToStatement + Sync + Send,
        R: FromPostgresRow;
}

#[async_trait]
impl DBTX for Transaction<'_> {
    async fn prepare(&self, query: &str) -> Result<Statement> {
        Ok(Transaction::prepare(self, query).await?)
    }

    async fn execute<T>(&self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<u64>
    where
        T: ?Sized + ToStatement + Sync + Send,
    {
        Ok(Transaction::execute(self, statement, params).await?)
    }

    async fn query_one<T, R>(&self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<R>
    where
        T: ?Sized + ToStatement + Sync + Send,
        R: FromPostgresRow,
    {
        let row = Transaction::query_one(self, statement, params).await?;
        Ok(FromPostgresRow::from_row(&row)?)
    }

    async fn query<T, R>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Box<dyn std::iter::Iterator<Item = crate::Result<R>>>>
    where
        T: ?Sized + ToStatement + Sync + Send,
        R: FromPostgresRow,
    {
        let rows = Transaction::query(self, statement, params).await?;
        let iter = rows
            .into_iter()
            .map(|row| Ok(FromPostgresRow::from_row(&row)?));
        Ok(Box::new(iter))
    }
}

#[async_trait]
impl DBTX for Client {
    async fn prepare(&self, query: &str) -> Result<Statement> {
        Ok(Client::prepare(self, query).await?)
    }

    async fn execute<T>(&self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<u64>
    where
        T: ?Sized + ToStatement + Sync + Send,
    {
        Ok(Client::execute(self, statement, params).await?)
    }

    async fn query_one<T, R>(&self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<R>
    where
        T: ?Sized + ToStatement + Sync + Send,
        R: FromPostgresRow,
    {
        let row = Client::query_one(self, statement, params).await?;
        Ok(FromPostgresRow::from_row(&row)?)
    }

    async fn query<T, R>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Box<dyn std::iter::Iterator<Item = crate::Result<R>>>>
    where
        T: ?Sized + ToStatement + Sync + Send,
        R: FromPostgresRow,
    {
        let rows = Client::query(self, statement, params).await?;
        let iter = rows
            .into_iter()
            .map(|row| Ok(FromPostgresRow::from_row(&row)?));
        Ok(Box::new(iter))
    }
}
