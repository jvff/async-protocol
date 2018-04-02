use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex, MutexGuard};

use futures::{Async, Poll, Stream};

use super::dispatcher::Dispatcher;
use super::message_with_id::MessageWithId;
use super::receiver::Receiver;

pub struct MultiplexDispatcher<T>
where
    T: Stream,
    T::Item: MessageWithId,
    <T::Item as MessageWithId>::Id: Eq + Hash,
{
    source: Arc<Mutex<T>>,
    queue: Arc<Mutex<HashMap<<T::Item as MessageWithId>::Id, T::Item>>>,
}

impl<T> MultiplexDispatcher<T>
where
    T: Stream,
    T::Item: MessageWithId,
    <T::Item as MessageWithId>::Id: Eq + Hash,
{
    pub fn new(source: T) -> Self {
        MultiplexDispatcher {
            source: Arc::new(Mutex::new(source)),
            queue: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn get_from_source(&self) -> Poll<(), T::Error> {
        let mut source = Self::lock(&self.source);

        if let Some(item) = try_ready!(source.poll()) {
            let id = item.id();
            let mut queue = Self::lock(&self.queue);

            queue.insert(id, item);

            while let Some(item) = try_ready!(source.poll()) {
                let id = item.id();
                queue.insert(id, item);
            }
        }

        Ok(Async::Ready(()))
    }

    fn remove_if_ready(
        &self,
        id: &<T::Item as MessageWithId>::Id,
    ) -> Option<T::Item> {
        let mut queue = Self::lock(&self.queue);

        queue.remove(id)
    }

    fn lock<I>(item: &Arc<Mutex<I>>) -> MutexGuard<I> {
        item.lock().expect(
            "a thread panicked while holding the MultiplexDispatcher locked",
        )
    }
}

impl<T> Dispatcher for MultiplexDispatcher<T>
where
    T: Stream,
    T::Item: MessageWithId,
    <T::Item as MessageWithId>::Id: Eq + Hash,
{
    type Item = T::Item;
    type Error = T::Error;
    type Id = <T::Item as MessageWithId>::Id;
    type Seed = Self::Id;

    fn spawn_receiver(arc_self: Arc<Self>, id: Self::Seed) -> Receiver<Self> {
        Receiver::new(arc_self, id)
    }

    fn poll(&self, id: &Self::Id) -> Poll<Self::Item, Self::Error> {
        if let Some(item) = self.remove_if_ready(id) {
            Ok(Async::Ready(item))
        } else {
            self.get_from_source()?;

            let item = self.remove_if_ready(id)
                .map(Async::Ready)
                .unwrap_or(Async::NotReady);

            Ok(item)
        }
    }
}
