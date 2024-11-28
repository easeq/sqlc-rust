use postgres::types::ToSql;
use postgres::{Client, Error, Row, Statement, ToStatement, Transaction};

pub trait DBTX {
    fn prepare(&mut self, query: &str) -> Result<Statement, Error>;
    fn execute<T>(&mut self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<u64, Error>
    where
        T: ?Sized + ToStatement + Sync + Send;
    fn query_one<T>(&mut self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<Row, Error>
    where
        T: ?Sized + ToStatement + Sync + Send;
    fn query<T>(
        &mut self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, Error>
    where
        T: ?Sized + ToStatement + Sync + Send;
}

impl DBTX for Transaction<'_> {
    fn prepare(&mut self, query: &str) -> Result<Statement, Error> {
        Transaction::prepare(self, query)
    }

    fn execute<T>(&mut self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<u64, Error>
    where
        T: ?Sized + ToStatement + Sync + Send,
    {
        Transaction::execute(self, statement, params)
    }

    fn query_one<T>(&mut self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<Row, Error>
    where
        T: ?Sized + ToStatement + Sync + Send,
    {
        Transaction::query_one(self, statement, params)
    }

    fn query<T>(&mut self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Row>, Error>
    where
        T: ?Sized + ToStatement + Sync + Send,
    {
        Transaction::query(self, statement, params)
    }
}

impl DBTX for Client {
    fn prepare(&mut self, query: &str) -> Result<Statement, Error> {
        Client::prepare(self, query)
    }

    fn execute<T>(&mut self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<u64, Error>
    where
        T: ?Sized + ToStatement + Sync + Send,
    {
        Client::execute(self, statement, params)
    }

    fn query_one<T>(&mut self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<Row, Error>
    where
        T: ?Sized + ToStatement + Sync + Send,
    {
        Client::query_one(self, statement, params)
    }

    fn query<T>(&mut self, statement: &T, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Row>, Error>
    where
        T: ?Sized + ToStatement + Sync + Send,
    {
        Client::query(self, statement, params)
    }
}
