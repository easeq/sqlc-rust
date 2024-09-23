use postgres::{Error, Row};

pub trait FromPostgresRow: Sized {
    fn from_row(row: &Row) -> Result<Self, Error>;
}

macro_rules! from_primitive {
    ($t:ty) => {
        impl FromPostgresRow for $t {
            fn from_row(row: &Row) -> Result<Self, Error> {
                Ok(row.get::<&str, $t>("0"))
            }
        }
    };
}

from_primitive!(bool);
from_primitive!(String);
from_primitive!(i16);
from_primitive!(i32);
from_primitive!(i64);
from_primitive!(f64);
