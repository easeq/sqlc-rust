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

pub type Result<T> = std::result::Result<T, Error>;
