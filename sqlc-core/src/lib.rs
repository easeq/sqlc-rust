use cfg_block::cfg_block;

#[cfg(all(feature = "with-postgres", feature = "with-tokio-postgres"))]
compile_error!(
            "features with-postgres and with-tokio-postgres are mutually exclusive and cannot be enabled together"
        );

#[cfg(all(not(feature = "with-postgres"), not(feature = "with-tokio-postgres")))]
compile_error!("one of with-postgres and with-tokio-postgres features needs to be enabled");

#[cfg(feature = "with-postgres")]
use postgres::{Error as PgError, Row};

#[cfg(feature = "with-tokio-postgres")]
use tokio_postgres::{Error as PgError, Row};

pub trait FromPostgresRow: Sized {
    fn from_row(row: &Row) -> Result<Self, PgError>;
}

macro_rules! from_primitive {
    ($t:ty) => {
        impl FromPostgresRow for $t {
            fn from_row(row: &Row) -> Result<Self, PgError> {
                Ok(row.try_get::<&str, $t>("0")?)
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

#[cfg(feature = "with-uuid-0_8")]
from_primitive!(uuid_0_8::Uuid);

#[cfg(feature = "with-uuid-1")]
from_primitive!(uuid_1::Uuid);

#[cfg(feature = "with-eui48-0_4")]
from_primitive!(eui48_04::MacAddress);

#[cfg(feature = "with-eui48-1")]
from_primitive!(eui48_1::MacAddress);

cfg_block! {
    #[cfg(feature = "with-cidr-0_2")] {
        from_primitive!(cidr_02::IpInet);
        from_primitive!(cidr_02::IpCidr);
    }

    #[cfg(feature = "with-time-0_2")] {
        from_primitive!(time_02::Time);
        from_primitive!(time_02::Date);
        from_primitive!(time_02::PrimitiveDateTime);
        from_primitive!(time_02::OffsetDateTime);
    }

    #[cfg(feature = "with-time-0_3")] {
        from_primitive!(time_03::Time);
        from_primitive!(time_03::Date);
        from_primitive!(time_03::PrimitiveDateTime);
        from_primitive!(time_03::OffsetDateTime);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "with-deadpool")]
    #[error("deadpool-postgres error: {0}")]
    DeadpoolError(#[from] deadpool_postgres::PoolError),

    #[cfg(feature = "with-postgres")]
    #[error("postgres error: {0}")]
    PostgresError(#[from] postgres::Error),

    #[cfg(feature = "with-tokio-postgres")]
    #[error("tokio-postgres error: {0}")]
    TokioPostgresError(#[from] tokio_postgres::Error),
}
