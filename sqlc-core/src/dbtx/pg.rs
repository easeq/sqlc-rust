use crate::{FromPostgresRow, Result};
use postgres::types::ToSql;
use postgres::{Client, Statement, ToStatement, Transaction};

pub trait DBTX {
    fn prepare(&mut self, query: &str) -> Result<Statement>;
    fn execute<T>(&mut self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<u64>
    where
        T: ?Sized + ToStatement + Sync + Send;
    fn query_one<T, R>(&mut self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<R>
    where
        T: ?Sized + ToStatement + Sync + Send,
        R: FromPostgresRow;
    fn query<T, R>(
        &mut self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Box<dyn std::iter::Iterator<Item = crate::Result<R>>>>
    where
        T: ?Sized + ToStatement + Sync + Send,
        R: FromPostgresRow;
}

impl DBTX for Transaction<'_> {
    fn prepare(&mut self, query: &str) -> Result<Statement> {
        Ok(Transaction::prepare(self, query)?)
    }

    fn execute<T>(&mut self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<u64>
    where
        T: ?Sized + ToStatement + Sync + Send,
    {
        Ok(Transaction::execute(self, statement, params)?)
    }

    fn query_one<T, R>(&mut self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<R>
    where
        T: ?Sized + ToStatement + Sync + Send,
        R: FromPostgresRow,
    {
        let row = Transaction::query_one(self, statement, params)?;
        Ok(FromPostgresRow::from_row(&row)?)
    }

    fn query<T, R>(
        &mut self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Box<dyn std::iter::Iterator<Item = crate::Result<R>>>>
    where
        T: ?Sized + ToStatement + Sync + Send,
        R: FromPostgresRow,
    {
        let rows = Transaction::query(self, statement, params)?;
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

    fn execute<T>(&mut self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<u64>
    where
        T: ?Sized + ToStatement + Sync + Send,
    {
        Ok(Client::execute(self, statement, params)?)
    }

    fn query_one<T, R>(&mut self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<R>
    where
        T: ?Sized + ToStatement + Sync + Send,
        R: FromPostgresRow,
    {
        let row = Client::query_one(self, statement, params)?;
        Ok(FromPostgresRow::from_row(&row)?)
    }

    fn query<T, R>(
        &mut self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Box<dyn std::iter::Iterator<Item = crate::Result<R>>>>
    where
        T: ?Sized + ToStatement + Sync + Send,
        R: FromPostgresRow,
    {
        let rows = Client::query(self, statement, params)?;
        let iter = rows
            .into_iter()
            .map(|row| Ok(FromPostgresRow::from_row(&row)?));
        Ok(Box::new(iter))
    }
}
