use std::sync::{Arc, Mutex};

use futures::future::Flatten;
use futures::stream::{SplitSink, SplitStream};
use futures::{Future, Sink, Stream};
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
    type Future = Flatten<
        ClientReceiver<
            's,
            FifoDispatcher<SplitStream<T>>,
            RequestSender<SplitSink<T>, ()>,
        >,
    >;

    fn call(&self, request: Self::Request) -> Self::Future {
        let sink = self.request_sink.clone();
        let dispatcher = &self.response_dispatcher;
        let send = RequestSender::new(sink, request, ());
        let receiver = ClientReceiver::new(dispatcher, send);

        receiver.flatten()
    }
}

#[cfg(test)]
mod tests {
    use futures::Async;
    use futures::sync::mpsc;

    use super::*;
    use tests::common::SinkStream;

    #[test]
    fn simple_operation() {
        let (mut in_tx, in_rx) = mpsc::channel(2);
        let (out_tx, mut out_rx) = mpsc::channel(2);
        let transport = SinkStream::new(out_tx, in_rx);

        let client = PipelineClient::new(transport);
        let client_service = &client;

        let first_request = "first request";
        let second_request = "second request";

        let first_call = client_service.call(first_request.to_string());
        let second_call = client_service.call(second_request.to_string());

        let first_response = "first response";
        let second_response = "second response";

        in_tx
            .try_send(first_response.to_string())
            .unwrap();
        in_tx
            .try_send(second_response.to_string())
            .unwrap();

        let calls = first_call.join(second_call);
        let (first_result, second_result) = calls.wait().unwrap();

        assert_eq!(receive(&mut out_rx), first_request);
        assert_eq!(receive(&mut out_rx), second_request);

        assert_eq!(first_result, first_response);
        assert_eq!(second_result, second_response);
    }

    #[test]
    fn inverted_join() {
        let (mut in_tx, in_rx) = mpsc::channel(2);
        let (out_tx, mut out_rx) = mpsc::channel(2);
        let transport = SinkStream::new(out_tx, in_rx);

        let client = PipelineClient::new(transport);
        let client_service = &client;

        let first_request = "first request";
        let second_request = "second request";

        let first_call = client_service.call(first_request.to_string());
        let second_call = client_service.call(second_request.to_string());

        let first_response = "first response";
        let second_response = "second response";

        in_tx
            .try_send(first_response.to_string())
            .unwrap();
        in_tx
            .try_send(second_response.to_string())
            .unwrap();

        let calls = second_call.join(first_call);
        let (first_result, second_result) = calls.wait().unwrap();

        assert_eq!(receive(&mut out_rx), second_request);
        assert_eq!(receive(&mut out_rx), first_request);

        assert_eq!(first_result, first_response);
        assert_eq!(second_result, second_response);
    }

    fn receive<S>(stream: &mut S) -> S::Item
    where
        S: Stream,
    {
        match stream.poll() {
            Ok(Async::Ready(Some(item))) => item,
            Ok(Async::Ready(None)) => {
                panic!("failed to receive item from stream: Stream is empty");
            }
            Ok(Async::NotReady) => {
                panic!("failed to receive item from stream: Not Ready");
            }
            Err(_) => {
                panic!("failed to receive item from stream: Error");
            }
        }
    }
}
