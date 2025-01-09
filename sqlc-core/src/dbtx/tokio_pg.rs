use crate::{AsPostgresParams, FromPostgresRow, Result};
use async_trait::async_trait;
use futures::stream::Stream;
use futures::Future;
use std::borrow::Borrow;
use std::pin::Pin;
use tokio_postgres::{Client, Statement, ToStatement, Transaction};

pub type BoxedFuture<'a, R> = Pin<Box<dyn Future<Output = Result<R>> + 'a>>;
pub type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + 'a>>;
pub type BatchStream<'a, R> = BoxStream<'a, BoxedFuture<'a, R>>;

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
    async fn batch_execute<'a, I, P>(
        &'a self,
        query: &'a str,
        arg_list: I,
    ) -> Result<BatchStream<()>>
    where
        I: IntoIterator + Send + 'a,
        I::Item: std::borrow::Borrow<P> + 'a,
        P: AsPostgresParams + Sync;
    async fn batch_one<'a, I, P, R>(
        &'a self,
        query: &'a str,
        arg_list: I,
    ) -> Result<BatchStream<R>>
    where
        I: IntoIterator + Send + 'a,
        I::Item: std::borrow::Borrow<P> + 'a,
        P: AsPostgresParams + Sync,
        R: FromPostgresRow + 'a;
    async fn batch_many<'a, I, P, R>(
        &'a self,
        query: &'a str,
        arg_list: I,
    ) -> Result<BatchStream<BoxStream<Result<R>>>>
    where
        I: IntoIterator + Send + 'a,
        I::Item: std::borrow::Borrow<P> + 'a,
        P: AsPostgresParams + Sync + 'a,
        R: FromPostgresRow + 'a;
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

    async fn batch_execute<'a, I, P>(
        &'a self,
        query: &'a str,
        arg_list: I,
    ) -> Result<BatchStream<()>>
    where
        I: IntoIterator + Send + 'a,
        I::Item: std::borrow::Borrow<P> + 'a,
        P: AsPostgresParams + Sync,
    {
        let stmt = DBTX::prepare(self, query).await?;
        let fut = move |item: <I as IntoIterator>::Item| -> BoxedFuture<()> {
            let stmt = stmt.clone();
            Box::pin(async move {
                let arg = item.borrow();
                DBTX::execute(self, &stmt, arg).await?;
                Ok(())
            })
        };
        Ok(Box::pin(futures::stream::iter(
            arg_list.into_iter().map(fut),
        )))
    }

    async fn batch_one<'a, I, P, R>(&'a self, query: &'a str, arg_list: I) -> Result<BatchStream<R>>
    where
        I: IntoIterator + Send + 'a,
        I::Item: std::borrow::Borrow<P> + 'a,
        P: AsPostgresParams + Sync,
        R: FromPostgresRow + 'a,
    {
        let stmt = DBTX::prepare(self, query).await?;
        let fut = move |item: <I as IntoIterator>::Item| -> BoxedFuture<R> {
            let stmt = stmt.clone();
            Box::pin(async move {
                let arg = item.borrow();
                let result = DBTX::query_one(self, &stmt, arg).await?;
                Ok(result)
            })
        };
        Ok(Box::pin(futures::stream::iter(
            arg_list.into_iter().map(fut),
        )))
    }

    async fn batch_many<'a, I, P, R>(
        &'a self,
        query: &'a str,
        arg_list: I,
    ) -> Result<BatchStream<BoxStream<Result<R>>>>
    where
        I: IntoIterator + Send + 'a,
        I::Item: std::borrow::Borrow<P> + 'a,
        P: AsPostgresParams + Sync + 'a,
        R: FromPostgresRow + 'a,
    {
        let stmt = DBTX::prepare(self, query).await?;
        let fut = move |item: <I as IntoIterator>::Item| -> BoxedFuture<BoxStream<Result<R>>> {
            let stmt = stmt.clone();
            Box::pin(async move {
                let arg = item.borrow();
                let result = DBTX::query(self, &stmt, arg).await?;
                let stream: BoxStream<Result<R>> = Box::pin(futures::stream::iter(result));
                Ok(stream)
            })
        };
        Ok(Box::pin(futures::stream::iter(
            arg_list.into_iter().map(fut),
        )))
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

    async fn batch_execute<'a, I, P>(
        &'a self,
        query: &'a str,
        arg_list: I,
    ) -> Result<BatchStream<()>>
    where
        I: IntoIterator + Send + 'a,
        I::Item: std::borrow::Borrow<P> + 'a,
        P: AsPostgresParams + Sync,
    {
        let stmt = DBTX::prepare(self, query).await?;
        let fut = move |item: <I as IntoIterator>::Item| -> BoxedFuture<()> {
            let stmt = stmt.clone();
            Box::pin(async move {
                let arg = item.borrow();
                DBTX::execute(self, &stmt, arg).await?;
                Ok(())
            })
        };
        Ok(Box::pin(futures::stream::iter(
            arg_list.into_iter().map(fut),
        )))
    }

    async fn batch_one<'a, I, P, R>(&'a self, query: &'a str, arg_list: I) -> Result<BatchStream<R>>
    where
        I: IntoIterator + Send + 'a,
        I::Item: std::borrow::Borrow<P> + 'a,
        P: AsPostgresParams + Sync,
        R: FromPostgresRow + 'a,
    {
        let stmt = DBTX::prepare(self, query).await?;
        let fut = move |item: <I as IntoIterator>::Item| -> BoxedFuture<R> {
            let stmt = stmt.clone();
            Box::pin(async move {
                let arg = item.borrow();
                let result = DBTX::query_one(self, &stmt, arg).await?;
                Ok(result)
            })
        };
        Ok(Box::pin(futures::stream::iter(
            arg_list.into_iter().map(fut),
        )))
    }

    async fn batch_many<'a, I, P, R>(
        &'a self,
        query: &'a str,
        arg_list: I,
    ) -> Result<BatchStream<BoxStream<Result<R>>>>
    where
        I: IntoIterator + Send + 'a,
        I::Item: std::borrow::Borrow<P> + 'a,
        P: AsPostgresParams + Sync + 'a,
        R: FromPostgresRow + 'a,
    {
        let stmt = DBTX::prepare(self, query).await?;
        let fut = move |item: <I as IntoIterator>::Item| -> BoxedFuture<BoxStream<Result<R>>> {
            let stmt = stmt.clone();
            Box::pin(async move {
                let arg = item.borrow();
                let result = DBTX::query(self, &stmt, arg).await?;
                let stream: BoxStream<Result<R>> = Box::pin(futures::stream::iter(result));
                Ok(stream)
            })
        };
        Ok(Box::pin(futures::stream::iter(
            arg_list.into_iter().map(fut),
        )))
    }
}
