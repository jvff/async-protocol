use std::sync::Arc;

use futures::{Future, Poll};

use super::dispatcher::Dispatcher;

pub struct Receiver<D>
where
    D: Dispatcher,
{
    dispatcher: Arc<D>,
    id: D::Id,
}

impl<D> Receiver<D>
where
    D: Dispatcher,
{
    pub fn new(dispatcher: Arc<D>, id: D::Id) -> Self {
        Receiver { dispatcher, id }
    }
}

impl<D> Future for Receiver<D>
where
    D: Dispatcher,
{
    type Item = D::Item;
    type Error = D::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.dispatcher.poll(&self.id)
    }
}
