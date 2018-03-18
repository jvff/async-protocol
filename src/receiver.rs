use futures::{Future, Poll};

use super::dispatcher::Dispatcher;

pub struct Receiver<'d, D>
where
    D: Dispatcher + 'd,
{
    dispatcher: &'d D,
    id: D::Id,
}

impl<'d, D> Receiver<'d, D>
where
    D: Dispatcher + 'd,
{
    pub fn new(dispatcher: &'d D, id: D::Id) -> Self {
        Receiver { dispatcher, id }
    }
}

impl<'d, D> Future for Receiver<'d, D>
where
    D: Dispatcher + 'd,
{
    type Item = D::Item;
    type Error = D::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.dispatcher.poll(&self.id)
    }
}
