use std::hash::Hash;
use std::sync::{Arc, Mutex};

use futures::future::Flatten;
use futures::stream::{SplitSink, SplitStream};
use futures::{Future, Sink, Stream};
use tokio_service::Service;

use super::client_error::ClientError;
use super::client_receiver::ClientReceiver;
use super::message_with_id::MessageWithId;
use super::multiplex_dispatcher::MultiplexDispatcher;
use super::request_sender::RequestSender;

pub struct MultiplexClient<T>
where
    T: Stream + Sink,
    T::Item: MessageWithId,
    T::SinkItem: MessageWithId<Id = <T::Item as MessageWithId>::Id>,
    <T::Item as MessageWithId>::Id: Eq + Hash,
{
    request_sink: Arc<Mutex<SplitSink<T>>>,
    response_dispatcher: MultiplexDispatcher<SplitStream<T>>,
}

impl<T> MultiplexClient<T>
where
    T: Stream + Sink,
    T::Item: MessageWithId,
    T::SinkItem: MessageWithId<Id = <T::Item as MessageWithId>::Id>,
    <T::Item as MessageWithId>::Id: Eq + Hash,
{
    pub fn new(transport: T) -> Self {
        let (outgoing, incoming) = transport.split();

        MultiplexClient {
            request_sink: Arc::new(Mutex::new(outgoing)),
            response_dispatcher: MultiplexDispatcher::new(incoming),
        }
    }
}

impl<'s, T> Service for &'s MultiplexClient<T>
where
    T: Stream + Sink,
    T::Item: MessageWithId,
    T::SinkItem: MessageWithId<Id = <T::Item as MessageWithId>::Id>,
    <T::Item as MessageWithId>::Id: Eq + Hash,
{
    type Request = T::SinkItem;
    type Response = T::Item;
    type Error = ClientError<T::Error, T::SinkError>;
    type Future = Flatten<
        ClientReceiver<
            's,
            MultiplexDispatcher<SplitStream<T>>,
            RequestSender<SplitSink<T>, <T::Item as MessageWithId>::Id>,
        >,
    >;

    fn call(&self, request: Self::Request) -> Self::Future {
        let id = request.id();
        let sink = self.request_sink.clone();
        let dispatcher = &self.response_dispatcher;
        let send = RequestSender::new(sink, request, id);
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

        let client = MultiplexClient::new(transport);
        let client_service = &client;

        let first_request = (79, "first request".to_owned());
        let second_request = (1094, "second request".to_owned());

        let first_call = client_service.call(first_request.clone());
        let second_call = client_service.call(second_request.clone());

        let first_response = (79, "first response".to_owned());
        let second_response = (1094, "second response".to_owned());

        in_tx.try_send(first_response.clone()).unwrap();
        in_tx.try_send(second_response.clone()).unwrap();

        let calls = first_call.join(second_call);
        let (first_result, second_result) = calls.wait().unwrap();

        assert_eq!(receive(&mut out_rx), first_request);
        assert_eq!(receive(&mut out_rx), second_request);

        assert_eq!(first_result, first_response);
        assert_eq!(second_result, second_response);
    }

    #[test]
    fn inverted_joins() {
        let (mut in_tx, in_rx) = mpsc::channel(2);
        let (out_tx, mut out_rx) = mpsc::channel(2);
        let transport = SinkStream::new(out_tx, in_rx);

        let client = MultiplexClient::new(transport);
        let client_service = &client;

        let first_request = (79, "first request".to_owned());
        let second_request = (1094, "second request".to_owned());

        let first_call = client_service.call(first_request.clone());
        let second_call = client_service.call(second_request.clone());

        let first_response = (79, "first response".to_owned());
        let second_response = (1094, "second response".to_owned());

        in_tx.try_send(first_response.clone()).unwrap();
        in_tx.try_send(second_response.clone()).unwrap();

        let calls = second_call.join(first_call);
        let (second_result, first_result) = calls.wait().unwrap();

        assert_eq!(receive(&mut out_rx), second_request);
        assert_eq!(receive(&mut out_rx), first_request);

        assert_eq!(first_result, first_response);
        assert_eq!(second_result, second_response);
    }

    #[test]
    fn inverted_responses() {
        let (mut in_tx, in_rx) = mpsc::channel(2);
        let (out_tx, mut out_rx) = mpsc::channel(2);
        let transport = SinkStream::new(out_tx, in_rx);

        let client = MultiplexClient::new(transport);
        let client_service = &client;

        let first_request = (79, "first request".to_owned());
        let second_request = (1094, "second request".to_owned());

        let first_call = client_service.call(first_request.clone());
        let second_call = client_service.call(second_request.clone());

        let first_response = (79, "first response".to_owned());
        let second_response = (1094, "second response".to_owned());

        in_tx.try_send(second_response.clone()).unwrap();
        in_tx.try_send(first_response.clone()).unwrap();

        let calls = first_call.join(second_call);
        let (first_result, second_result) = calls.wait().unwrap();

        assert_eq!(receive(&mut out_rx), first_request);
        assert_eq!(receive(&mut out_rx), second_request);

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
