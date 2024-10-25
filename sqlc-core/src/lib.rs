use cfg_block::cfg_block;
use std::collections::HashMap;

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

#[cfg(feature = "with-deadpool")]
pub trait BatchResult: futures::Stream + Send {
    type Param;

    fn stmt(&self) -> tokio_postgres::Statement;

    fn current_item(&self) -> Option<Self::Param>;

    fn pool(&self) -> deadpool_postgres::Pool;

    fn inc_index(self: std::pin::Pin<&mut Self>);

    fn set_thunk(
        self: std::pin::Pin<&mut Self>,
        thunk: std::pin::Pin<
            Box<dyn futures::Future<Output = Option<<Self as futures::Stream>::Item>> + Send>,
        >,
    );

    fn thunk(
        self: std::pin::Pin<&mut Self>,
        arg: Self::Param,
        stmt: tokio_postgres::Statement,
        pool: deadpool_postgres::Pool,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Option<<Self as futures::Stream>::Item>> + Send>,
    >;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<<Self as futures::Stream>::Item>> {
        if let Some(arg) = self.current_item() {
            let stmt = self.stmt();
            let pool = self.pool();
            let mut fut = self.as_mut().thunk(arg, stmt, pool);

            match fut.as_mut().poll(cx) {
                std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
                std::task::Poll::Ready(res) => {
                    self.inc_index();
                    std::task::Poll::Ready(res)
                }
                std::task::Poll::Pending => {
                    self.set_thunk(fut);
                    std::task::Poll::Pending
                }
            }
        } else {
            std::task::Poll::Ready(None)
        }
    }
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
from_primitive!(HashMap<String, Option<String>>);

#[cfg(feature = "with-bit-vec-0_6")]
from_primitive!(bit_vec_06::BitVec);

#[cfg(feature = "with-uuid-0_8")]
from_primitive!(uuid_0_8::Uuid);

#[cfg(feature = "with-uuid-1")]
from_primitive!(uuid_1::Uuid);

#[cfg(feature = "with-eui48-0_4")]
from_primitive!(eui48_04::MacAddress);

#[cfg(feature = "with-eui48-1")]
from_primitive!(eui48_1::MacAddress);

#[cfg(feature = "with-serde_json-1")]
from_primitive!(serde_json_1::Value);

cfg_block! {
    #[cfg(feature = "with-cidr-0_2")] {
        from_primitive!(cidr_02::IpInet);
        from_primitive!(cidr_02::IpCidr);
    }

    #[cfg(feature = "with-geo-types-0_6")] {
        from_primitive!(geo_types_06::Point);
        from_primitive!(geo_types_06::Rect);
        from_primitive!(geo_types_06::LineString);
    }

    #[cfg(feature = "with-geo-types-0_7")] {
        from_primitive!(geo_types_0_7::Point);
        from_primitive!(geo_types_0_7::Rect);
        from_primitive!(geo_types_0_7::LineString);
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
