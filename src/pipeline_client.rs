use std::sync::{Arc, Mutex};

use futures::{Future, Sink, Stream};
use futures::future::Flatten;
use futures::stream::{SplitSink, SplitStream};
use tokio_service::Service;

use super::client_error::ClientError;
use super::client_receiver::ClientReceiver;
use super::fifo_dispatcher::FifoDispatcher;
use super::request_sender::RequestSender;

pub struct PipelineClient<T>
where
    T: Stream + Sink,
{
    request_sink: Arc<Mutex<SplitSink<T>>>,
    response_dispatcher: FifoDispatcher<SplitStream<T>>,
}

impl<T> PipelineClient<T>
where
    T: Stream + Sink,
{
    pub fn new(transport: T) -> Self {
        let (outgoing, incoming) = transport.split();

        PipelineClient {
            request_sink: Arc::new(Mutex::new(outgoing)),
            response_dispatcher: FifoDispatcher::new(incoming),
        }
    }
}

impl<'s, T> Service for &'s PipelineClient<T>
where
    T: Stream + Sink,
{
    type Request = T::SinkItem;
    type Response = T::Item;
    type Error = ClientError<T::Error, T::SinkError>;
    type Future =
        Flatten<
            ClientReceiver<
                's,
                FifoDispatcher<SplitStream<T>>,
                RequestSender<SplitSink<T>>,
            >,
        >;

    fn call(&self, request: Self::Request) -> Self::Future {
        let sink = self.request_sink.clone();
        let dispatcher = &self.response_dispatcher;
        let send = RequestSender::new(sink, request);
        let receiver = ClientReceiver::new(dispatcher, send);

        receiver.flatten()
    }
}
