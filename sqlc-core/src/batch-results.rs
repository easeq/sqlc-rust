#[pin_project::pin_project]
pub struct BatchResults<Callback, Fut, P> {
    __pool: deadpool_postgres::Pool,
    __stmt: tokio_postgres::Statement,
    __index: usize,

    __items: Vec<P>,
    __total: usize,

    __callback: Callback,
    #[pin]
    __thunk: Option<Fut>,
}

impl<Callback, Fut, P> BatchResults<Callback, Fut, P> {
    pub fn new(
        __pool: deadpool_postgres::Pool,
        __items: Vec<P>,
        __stmt: tokio_postgres::Statement,
        __callback: Callback,
    ) -> Self {
        Self {
            __pool,
            __total: __items.len(),
            __items,
            __stmt,
            __callback,
            __index: 0,
            __thunk: None,
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
        if this.__index < this.__total {
            let arg = this.__items[*this.__index].clone();
            let stmt = this.__stmt.clone();
            let pool = this.__pool.clone();

            if this.__thunk.is_none() {
                this.__thunk.as_mut().set(Some((this.__callback)(
                    pool.clone(),
                    stmt.clone(),
                    arg.clone(),
                )));
            }

            let mut fut = this.__thunk.as_mut().as_pin_mut().unwrap();
            match fut.as_mut().poll(cx) {
                std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Some(Err(e))),
                std::task::Poll::Ready(Ok(res)) => {
                    *this.__index += 1;
                    this.__thunk.set(None);
                    std::task::Poll::Ready(Some(Ok(res)))
                }
                std::task::Poll::Pending => std::task::Poll::Pending,
            }
        } else {
            std::task::Poll::Ready(None)
        }
    }
}
