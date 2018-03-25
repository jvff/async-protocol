use std::sync::{Arc, Mutex};

use futures::{Async, AsyncSink, Future, Poll, Sink};

pub struct RequestSender<T>
where
    T: Sink,
{
    sink: Arc<Mutex<T>>,
    request: Option<T::SinkItem>,
}

impl<T> RequestSender<T>
where
    T: Sink,
{
    pub fn new(sink: Arc<Mutex<T>>, request: T::SinkItem) -> Self {
        RequestSender {
            sink,
            request: Some(request),
        }
    }
}

impl<T> Future for RequestSender<T>
where
    T: Sink,
{
    type Item = ();
    type Error = T::SinkError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut sink = self.sink
            .lock()
            .expect("a thread panicked while holding RequestSender locked");

        if let Some(request) = self.request.take() {
            match sink.start_send(request)? {
                AsyncSink::Ready => (),
                AsyncSink::NotReady(request) => {
                    self.request = Some(request);
                    return Ok(Async::NotReady);
                }
            }
        }

        sink.poll_complete()
    }
}
