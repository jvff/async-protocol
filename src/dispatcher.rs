use futures::Poll;

use super::receiver::Receiver;

pub trait Dispatcher {
    type Item;
    type Error;
    type Id;

    fn spawn_receiver(&self) -> Receiver<Self>
    where
        Self: Sized;

    fn poll(&self, id: &Self::Id) -> Poll<Self::Item, Self::Error>;
}
