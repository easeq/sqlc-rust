use cfg_block::cfg_block;

#[cfg(all(feature = "with-postgres", feature = "with-tokio-postgres"))]
compile_error!(
    "features with-postgres and with-tokio-postgres are mutually exclusive and cannot be enabled together"
);

#[cfg(all(not(feature = "with-postgres"), not(feature = "with-tokio-postgres")))]
compile_error!("one of with-postgres and with-tokio-postgres features needs to be enabled");

mod error;
mod from_postgres_row;

pub use error::*;
pub use from_postgres_row::*;

cfg_block! {
    #[cfg(feature = "with-deadpool")] {
        mod batch_results;
        pub use batch_results::*;
    }
}
