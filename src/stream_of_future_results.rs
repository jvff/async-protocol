use futures::Future;
use futures::stream::{FuturesOrdered, FuturesUnordered, Stream};

pub trait StreamOfFutureResults<F>:
    Stream<Item = F::Item, Error = F::Error>
where
    F: Future,
{
    fn new() -> Self;
    fn push(&mut self, future: F);
}

impl<F> StreamOfFutureResults<F> for FuturesOrdered<F>
where
    F: Future,
{
    fn new() -> Self {
        FuturesOrdered::new()
    }

    fn push(&mut self, future: F) {
        FuturesOrdered::push(self, future);
    }
}

impl<F> StreamOfFutureResults<F> for FuturesUnordered<F>
where
    F: Future,
{
    fn new() -> Self {
        FuturesUnordered::new()
    }

    fn push(&mut self, future: F) {
        FuturesUnordered::push(self, future);
    }
}
