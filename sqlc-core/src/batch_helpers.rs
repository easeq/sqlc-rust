use crate::{AsPostgresParams, FromPostgresRow, Result, DBTX};
use std::borrow::Borrow;

pub async fn batch_many<'a, I, C, P, R>(
    client: &'a C,
    query: &'a str,
    arg_list: I,
) -> Result<
    impl futures::Stream<
            Item = impl futures::Future<Output = Result<impl futures::Stream<Item = Result<R>>>> + 'a,
        > + 'a,
>
where
    C: DBTX,
    I: IntoIterator + 'a,
    I::Item: std::borrow::Borrow<P> + 'a,
    P: AsPostgresParams + Sync + 'a,
    R: FromPostgresRow,
{
    let stmt = client.prepare(query).await?;
    let fut = move |item: <I as IntoIterator>::Item| {
        let stmt = stmt.clone();
        Box::pin(async move {
            let arg = item.borrow();
            let result = client.query(&stmt, arg).await?;
            Ok(futures::stream::iter(result))
        })
    };
    Ok(futures::stream::iter(arg_list.into_iter().map(fut)))
}

pub async fn batch_one<'a, I, C, P, R>(
    client: &'a C,
    query: &'a str,
    arg_list: I,
) -> Result<impl futures::Stream<Item = impl futures::Future<Output = Result<R>> + 'a> + 'a>
where
    C: DBTX,
    I: IntoIterator + 'a,
    I::Item: std::borrow::Borrow<P> + 'a,
    P: AsPostgresParams + Sync + 'a,
    R: FromPostgresRow,
{
    let stmt = client.prepare(query).await?;
    let fut = move |item: <I as IntoIterator>::Item| {
        let stmt = stmt.clone();
        Box::pin(async move {
            let arg = item.borrow();
            let result = client.query_one(&stmt, arg).await?;
            Ok(result)
        })
    };
    Ok(futures::stream::iter(arg_list.into_iter().map(fut)))
}

pub async fn batch_execute<'a, I, C, P>(
    client: &'a C,
    query: &'a str,
    arg_list: I,
) -> Result<impl futures::Stream<Item = impl futures::Future<Output = Result<()>> + 'a> + 'a>
where
    C: DBTX,
    I: IntoIterator + 'a,
    I::Item: std::borrow::Borrow<P> + 'a,
    P: AsPostgresParams + Sync + 'a,
{
    let stmt = client.prepare(query).await?;
    let fut = move |item: <I as IntoIterator>::Item| {
        let stmt = stmt.clone();
        Box::pin(async move {
            let arg = item.borrow();
            client.execute(&stmt, arg).await?;
            Ok(())
        })
    };
    Ok(futures::stream::iter(arg_list.into_iter().map(fut)))
}
