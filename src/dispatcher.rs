use futures::Poll;

pub trait Dispatcher {
    type Item;
    type Error;
    type Id;

    fn poll(&self, id: &Self::Id) -> Poll<Self::Item, Self::Error>;
}
