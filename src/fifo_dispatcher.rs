use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::{Async, Poll, Stream};

use super::delayed_add::DelayedAdd;
use super::dispatcher::Dispatcher;
use super::ready_queue::ReadyQueue;
use super::receiver::Receiver;

pub struct FifoDispatcher<T>
where
    T: Stream,
{
    source: Arc<Mutex<T>>,
    queue: Arc<Mutex<ReadyQueue<T::Item>>>,
    latest_ready_id: AtomicUsize,
    new_id: AtomicUsize,
}

impl<T> FifoDispatcher<T>
where
    T: Stream,
{
    pub fn new(source: T) -> Self {
        FifoDispatcher {
            source: Arc::new(Mutex::new(source)),
            queue: Arc::new(Mutex::new(ReadyQueue::new())),
            latest_ready_id: AtomicUsize::new(0),
            new_id: AtomicUsize::new(0),
        }
    }

    fn pop_if_ready(&self, id: usize) -> Option<T::Item> {
        if id < self.latest_ready_id.load(Ordering::Relaxed) {
            Some(self.lock_queue().pop(id))
        } else {
            None
        }
    }

    fn get_from_source(&self) -> Poll<(), T::Error> {
        let mut source = self.source.lock().expect(
            "a thread panicked while holding the FifoDispatcher locked",
        );

        if let Some(item) = try_ready!(source.poll()) {
            let mut update_latest_ready_id =
                DelayedAdd::new(&self.latest_ready_id);
            let mut queue = self.lock_queue();

            queue.push(item);
            update_latest_ready_id.increment();

            while let Some(item) = try_ready!(source.poll()) {
                queue.push(item);
                update_latest_ready_id.increment();
            }
        }

        Ok(Async::Ready(()))
    }

    fn lock_queue(&self) -> MutexGuard<ReadyQueue<T::Item>> {
        self.queue
            .lock()
            .expect("a thread panicked while holding the FifoDispatcher locked")
    }
}

impl<T> Dispatcher for FifoDispatcher<T>
where
    T: Stream,
{
    type Item = T::Item;
    type Error = T::Error;
    type Id = usize;

    fn spawn_receiver(&self) -> Receiver<Self> {
        Receiver::new(
            &self,
            self.new_id.fetch_add(1, Ordering::Relaxed),
        )
    }

    fn poll(&self, id: &Self::Id) -> Poll<Self::Item, Self::Error> {
        if let Some(item) = self.pop_if_ready(*id) {
            Ok(Async::Ready(item))
        } else {
            self.get_from_source()?;

            let item = self.pop_if_ready(*id)
                .map(Async::Ready)
                .unwrap_or(Async::NotReady);

            Ok(item)
        }
    }
}
