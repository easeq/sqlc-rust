use postgres::{Error, Row};

pub trait FromPostgresRow: Sized {
    fn from_row(row: &Row) -> Result<Self, Error>;
}
