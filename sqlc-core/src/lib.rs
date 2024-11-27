use cfg_block::cfg_block;

#[cfg(all(feature = "with-postgres", feature = "with-tokio-postgres"))]
compile_error!(
    "features with-postgres and with-tokio-postgres are mutually exclusive and cannot be enabled together"
);

#[cfg(all(not(feature = "with-postgres"), not(feature = "with-tokio-postgres")))]
compile_error!("one of with-postgres and with-tokio-postgres features needs to be enabled");

mod dbtx;
mod error;
mod from_postgres_row;

pub use error::*;
pub use from_postgres_row::*;
pub use sqlc_derive::FromPostgresRow;

cfg_block! {
    #[cfg(feature = "with-postgres")] {
        pub use dbtx::pg::*;
    }

    #[cfg(feature = "with-tokio-postgres")] {
        pub use dbtx::tokio_pg::*;
    }
}
