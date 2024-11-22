use crate::db::Queries;

pub type TxCallback = Box<
    dyn Fn(
            Queries,
        ) -> std::pin::Pin<
            Box<dyn futures::Future<Output = std::result::Result<(), sqlc_core::Error>> + Send>,
        > + Send
        + Sync,
>;

pub(crate) struct Store {
    queries: crate::db::Queries,
}

impl Store {
    pub fn new(queries: Queries) -> Self {
        Self { queries }
    }

    pub async fn exec_tx(&self, cb: TxCallback) -> Result<(), sqlc_core::Error> {
        let transaction = self.queries.client().await.transaction().await.unwrap();

        // cb(self.queries.clone())

        Ok(())
    }
}
