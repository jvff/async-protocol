use futures::{Poll, Sink, StartSend, Stream};

pub struct SinkStream<I, O>
where
    I: Stream,
    O: Sink,
{
    sink: O,
    stream: I,
}

impl<I, O> SinkStream<I, O>
where
    I: Stream,
    O: Sink,
{
    pub fn new(sink: O, stream: I) -> Self {
        SinkStream { sink, stream }
    }
}

impl<I, O> Stream for SinkStream<I, O>
where
    I: Stream,
    O: Sink,
{
    type Item = I::Item;
    type Error = I::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.stream.poll()
    }
}

impl<I, O> Sink for SinkStream<I, O>
where
    I: Stream,
    O: Sink,
{
    type SinkItem = O::SinkItem;
    type SinkError = O::SinkError;

    fn start_send(
        &mut self,
        item: Self::SinkItem,
    ) -> StartSend<Self::SinkItem, Self::SinkError> {
        self.sink.start_send(item)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.sink.poll_complete()
    }
}
