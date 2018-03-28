use std::sync::{Arc, Mutex};

use futures::{Async, AsyncSink, Future, Poll, Sink};

pub struct RequestSender<O, S>
where
    O: Sink,
{
    sink: Arc<Mutex<O>>,
    request: Option<O::SinkItem>,
    seed: Option<S>,
}

impl<O, S> RequestSender<O, S>
where
    O: Sink,
{
    pub fn new(sink: Arc<Mutex<O>>, request: O::SinkItem, seed: S) -> Self {
        RequestSender {
            sink,
            request: Some(request),
            seed: Some(seed),
        }
    }
}

impl<O, S> Future for RequestSender<O, S>
where
    O: Sink,
{
    type Item = S;
    type Error = O::SinkError;

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

        try_ready!(sink.poll_complete());

        let seed = self.seed
            .take()
            .expect("Request sender polled after it had completed");

        Ok(Async::Ready(seed))
    }
}
