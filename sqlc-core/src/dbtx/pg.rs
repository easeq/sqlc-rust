use crate::{AsPostgresParams, FromPostgresRow, Result};
use postgres::{Client, Statement, ToStatement, Transaction};

pub trait DBTX {
    fn prepare(&mut self, query: &str) -> Result<Statement>;
    fn execute<T, P>(&mut self, statement: &T, params: P) -> Result<u64>
    where
        T: ?Sized + ToStatement + Sync + Send,
        P: AsPostgresParams + Send + Sync;
    fn query_one<T, P, R>(&mut self, statement: &T, params: P) -> Result<R>
    where
        T: ?Sized + ToStatement + Sync + Send,
        P: AsPostgresParams + Send + Sync,
        R: FromPostgresRow;
    fn query<T, P, R>(
        &mut self,
        statement: &T,
        params: P,
    ) -> Result<Box<dyn std::iter::Iterator<Item = crate::Result<R>>>>
    where
        T: ?Sized + ToStatement + Sync + Send,
        P: AsPostgresParams + Send + Sync,
        R: FromPostgresRow;
}

impl DBTX for Transaction<'_> {
    fn prepare(&mut self, query: &str) -> Result<Statement> {
        Ok(Transaction::prepare(self, query)?)
    }

    fn execute<T, P>(&mut self, statement: &T, params: P) -> Result<u64>
    where
        T: ?Sized + ToStatement + Sync + Send,
        P: AsPostgresParams + Send + Sync,
    {
        Ok(Transaction::execute(self, statement, &params.as_params())?)
    }

    fn query_one<T, P, R>(&mut self, statement: &T, params: P) -> Result<R>
    where
        T: ?Sized + ToStatement + Sync + Send,
        P: AsPostgresParams + Send + Sync,
        R: FromPostgresRow,
    {
        let row = Transaction::query_one(self, statement, &params.as_params())?;
        Ok(FromPostgresRow::from_row(&row)?)
    }

    fn query<T, P, R>(
        &mut self,
        statement: &T,
        params: P,
    ) -> Result<Box<dyn std::iter::Iterator<Item = crate::Result<R>>>>
    where
        T: ?Sized + ToStatement + Sync + Send,
        P: AsPostgresParams + Send + Sync,
        R: FromPostgresRow,
    {
        let rows = Transaction::query(self, statement, &params.as_params())?;
        let iter = rows
            .into_iter()
            .map(|row| Ok(FromPostgresRow::from_row(&row)?));
        Ok(Box::new(iter))
    }
}

impl DBTX for Client {
    fn prepare(&mut self, query: &str) -> Result<Statement> {
        Ok(Client::prepare(self, query)?)
    }

    fn execute<T, P>(&mut self, statement: &T, params: P) -> Result<u64>
    where
        T: ?Sized + ToStatement + Sync + Send,
        P: AsPostgresParams + Send + Sync,
    {
        Ok(Client::execute(self, statement, &params.as_params())?)
    }

    fn query_one<T, P, R>(&mut self, statement: &T, params: P) -> Result<R>
    where
        T: ?Sized + ToStatement + Sync + Send,
        P: AsPostgresParams + Send + Sync,
        R: FromPostgresRow,
    {
        let row = Client::query_one(self, statement, &params.as_params())?;
        Ok(FromPostgresRow::from_row(&row)?)
    }

    fn query<T, P, R>(
        &mut self,
        statement: &T,
        params: P,
    ) -> Result<Box<dyn std::iter::Iterator<Item = crate::Result<R>>>>
    where
        T: ?Sized + ToStatement + Sync + Send,
        P: AsPostgresParams + Send + Sync,
        R: FromPostgresRow,
    {
        let rows = Client::query(self, statement, &params.as_params())?;
        let iter = rows
            .into_iter()
            .map(|row| Ok(FromPostgresRow::from_row(&row)?));
        Ok(Box::new(iter))
    }
}
