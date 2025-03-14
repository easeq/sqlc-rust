#[cfg(all(feature = "with-postgres", feature = "with-tokio-postgres"))]
compile_error!(
    "features with-postgres and with-tokio-postgres are mutually exclusive and cannot be enabled together"
);

#[cfg(all(not(feature = "with-postgres"), not(feature = "with-tokio-postgres")))]
compile_error!("one of with-postgres and with-tokio-postgres features needs to be enabled");

#[cfg(all(
    not(feature = "with-rust_decimal-postgres"),
    not(feature = "with-rust_decimal-tokio-postgres")
))]
compile_error!("one of with-rust_decimal-postgres and with-rust_decimal-tokio-postgres features needs to be enabled");

mod dbtx;
mod error;
mod postgres_params;
mod postgres_row;

pub use error::*;
pub use postgres_params::*;
pub use postgres_row::*;
pub use sqlc_derive::PostgresParams;
pub use sqlc_derive::PostgresRow;

#[cfg(feature = "with-bit-vec-0_6")]
pub mod bit_vec {
    pub use bit_vec_06::*;
}

#[cfg(any(feature = "with-uuid-0_8", feature = "with-uuid-1"))]
pub mod uuid {
    #[cfg(feature = "with-uuid-0_8")]
    pub use uuid_0_8::*;

    #[cfg(feature = "with-uuid-1")]
    pub use uuid_1::*;
}

#[cfg(any(feature = "with-eui48-0_4", feature = "with-eui48-1"))]
pub mod eui48 {
    #[cfg(feature = "with-eui48-0_4")]
    pub use eui48_04::*;

    #[cfg(feature = "with-eui48-1")]
    pub use eui48_1::*;
}

#[cfg(feature = "with-serde_json-1")]
pub mod serde_json {
    pub use serde_json_1::*;
}

#[cfg(feature = "with-cidr-0_2")]
pub mod cidr {
    pub use cidr_02::*;
}

#[cfg(any(feature = "with-geo-types-0_6", feature = "with-geo-types-0_7"))]
pub mod geo_types {
    #[cfg(feature = "with-geo-types-0_6")]
    pub use geo_types_06::*;

    #[cfg(feature = "with-geo-types-0_7")]
    pub use geo_types_0_7::*;
}

#[cfg(any(feature = "with-time-0_2", feature = "with-time-0_3"))]
pub mod time {
    #[cfg(feature = "with-time-0_2")]
    pub use time_02::*;

    #[cfg(feature = "with-time-0_3")]
    pub use time_03::*;
}

#[cfg(any(
    feature = "with-rust_decimal-postgres",
    feature = "with-rust_decimal-tokio-postgres"
))]
pub mod rust_decimal {
    #[cfg(feature = "with-rust_decimal-postgres")]
    pub use rust_decimal_postgres::prelude::*;

    #[cfg(feature = "with-rust_decimal-tokio-postgres")]
    pub use rust_decimal_tokio_postgres::prelude::*;
}

#[cfg(feature = "with-postgres")]
pub use dbtx::pg::*;

#[cfg(feature = "with-tokio-postgres")]
pub use dbtx::tokio_pg::*;
