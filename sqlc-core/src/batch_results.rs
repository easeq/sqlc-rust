#[pin_project::pin_project]
pub struct BatchResults<Callback, Fut, P> {
    pool: deadpool_postgres::Pool,
    stmt: tokio_postgres::Statement,
    index: usize,

    items: Vec<P>,
    total: usize,

    callback: Callback,
    #[pin]
    thunk: Option<Fut>,
}

impl<Callback, Fut, P> BatchResults<Callback, Fut, P> {
    pub fn new(
        pool: deadpool_postgres::Pool,
        items: Vec<P>,
        stmt: tokio_postgres::Statement,
        callback: Callback,
    ) -> Self {
        Self {
            pool,
            total: items.len(),
            items,
            stmt,
            callback,
            index: 0,
            thunk: None,
        }
    }
}

impl<Callback, Fut, P, R> futures::Stream for BatchResults<Callback, Fut, P>
where
    Callback: Fn(deadpool_postgres::Pool, tokio_postgres::Statement, P) -> Fut,
    Fut: futures::Future<Output = Result<R, crate::Error>> + Send + 'static,
    P: Clone + 'static,
    R: 'static,
{
    type Item = Result<R, crate::Error>;
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();
        if this.index < this.total {
            let arg = this.items[*this.index].clone();
            let stmt = this.stmt.clone();
            let pool = this.pool.clone();

            if this.thunk.is_none() {
                this.thunk
                    .as_mut()
                    .set(Some((this.callback)(pool, stmt, arg)));
            }

            let mut fut = this.thunk.as_mut().as_pin_mut().unwrap();
            match fut.as_mut().poll(cx) {
                std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Some(Err(e))),
                std::task::Poll::Ready(Ok(res)) => {
                    *this.index += 1;
                    this.thunk.set(None);
                    std::task::Poll::Ready(Some(Ok(res)))
                }
                std::task::Poll::Pending => std::task::Poll::Pending,
            }
        } else {
            std::task::Poll::Ready(None)
        }
    }
}
