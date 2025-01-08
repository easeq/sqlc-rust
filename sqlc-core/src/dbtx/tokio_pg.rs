use crate::{AsPostgresParams, FromPostgresRow, Result};
use async_trait::async_trait;
use tokio_postgres::{Client, Statement, ToStatement, Transaction};

#[async_trait]
pub trait DBTX: Send + Sync {
    async fn prepare(&self, query: &str) -> Result<Statement>;
    async fn execute<T, P>(&self, statement: &T, params: P) -> Result<u64>
    where
        T: ?Sized + ToStatement + Sync + Send,
        P: AsPostgresParams + Send + Sync;
    async fn query_one<T, P, R>(&self, statement: &T, params: P) -> Result<R>
    where
        T: ?Sized + ToStatement + Sync + Send,
        P: AsPostgresParams + Send + Sync,
        R: FromPostgresRow;
    async fn query<T, P, R>(
        &self,
        statement: &T,
        params: P,
    ) -> Result<Box<dyn std::iter::Iterator<Item = crate::Result<R>>>>
    where
        T: ?Sized + ToStatement + Sync + Send,
        P: AsPostgresParams + Send + Sync,
        R: FromPostgresRow;
}

#[async_trait]
impl DBTX for Transaction<'_> {
    async fn prepare(&self, query: &str) -> Result<Statement> {
        Ok(Transaction::prepare(self, query).await?)
    }

    async fn execute<T, P>(&self, statement: &T, params: P) -> Result<u64>
    where
        T: ?Sized + ToStatement + Sync + Send,
        P: AsPostgresParams + Send + Sync,
    {
        Ok(Transaction::execute(self, statement, &params.as_params()).await?)
    }

    async fn query_one<T, P, R>(&self, statement: &T, params: P) -> Result<R>
    where
        T: ?Sized + ToStatement + Sync + Send,
        P: AsPostgresParams + Send + Sync,
        R: FromPostgresRow,
    {
        let row = Transaction::query_one(self, statement, &params.as_params()).await?;
        Ok(FromPostgresRow::from_row(&row)?)
    }

    async fn query<T, P, R>(
        &self,
        statement: &T,
        params: P,
    ) -> Result<Box<dyn std::iter::Iterator<Item = crate::Result<R>>>>
    where
        T: ?Sized + ToStatement + Sync + Send,
        P: AsPostgresParams + Send + Sync,
        R: FromPostgresRow,
    {
        let rows = Transaction::query(self, statement, &params.as_params()).await?;
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

    async fn execute<T, P>(&self, statement: &T, params: P) -> Result<u64>
    where
        T: ?Sized + ToStatement + Sync + Send,
        P: AsPostgresParams + Send + Sync,
    {
        Ok(Client::execute(self, statement, &params.as_params()).await?)
    }

    async fn query_one<T, P, R>(&self, statement: &T, params: P) -> Result<R>
    where
        T: ?Sized + ToStatement + Sync + Send,
        P: AsPostgresParams + Send + Sync,
        R: FromPostgresRow,
    {
        let row = Client::query_one(self, statement, &params.as_params()).await?;
        Ok(FromPostgresRow::from_row(&row)?)
    }

    async fn query<T, P, R>(
        &self,
        statement: &T,
        params: P,
    ) -> Result<Box<dyn std::iter::Iterator<Item = crate::Result<R>>>>
    where
        T: ?Sized + ToStatement + Sync + Send,
        P: AsPostgresParams + Send + Sync,
        R: FromPostgresRow,
    {
        let rows = Client::query(self, statement, &params.as_params()).await?;
        let iter = rows
            .into_iter()
            .map(|row| Ok(FromPostgresRow::from_row(&row)?));
        Ok(Box::new(iter))
    }
}
