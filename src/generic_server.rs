use futures::sink::SendAll;
use futures::stream::{Fuse, SplitSink, SplitStream};
use futures::sync::mpsc;
use futures::{Async, Future, Poll, Sink, Stream};
use tokio_service::Service;

use super::map_to_server_send_error::MapToServerSendError;
use super::server_error::ServerError;
use super::stream_of_future_results::StreamOfFutureResults;

pub type ServerErrorAlias<S: Service, T: Stream + Sink> =
    ServerError<T::Error, T::SinkError, S::Error>;

pub struct GenericServer<S, T, H>
where
    S: Service,
    T: Stream<Item = S::Request> + Sink<SinkItem = S::Response>,
    H: StreamOfFutureResults<S::Future>,
{
    service: S,
    incoming_requests: Fuse<SplitStream<T>>,
    active_requests: H,
    response_queue: mpsc::UnboundedSender<T::SinkItem>,
    response_sender: SendAll<
        MapToServerSendError<SplitSink<T>, ServerErrorAlias<S, T>>,
        mpsc::UnboundedReceiver<T::SinkItem>,
    >,
    no_more_requests: bool,
}

impl<S, T, H> GenericServer<S, T, H>
where
    S: Service,
    T: Stream<Item = S::Request> + Sink<SinkItem = S::Response>,
    H: StreamOfFutureResults<S::Future>,
{
    pub fn new(service: S, transport: T) -> Self {
        let (outgoing_responses, incoming_requests) = transport.split();
        let (response_queue, queued_responses) = mpsc::unbounded();

        let outgoing_responses = MapToServerSendError::from(outgoing_responses);
        let incoming_requests = incoming_requests.fuse();
        let active_requests = H::new();
        let response_sender = outgoing_responses.send_all(queued_responses);

        GenericServer {
            service,
            incoming_requests,
            active_requests,
            response_queue,
            response_sender,
            no_more_requests: false,
        }
    }

    fn poll_responses(&mut self) -> Result<(), ServerErrorAlias<S, T>> {
        loop {
            let next_response = self.active_requests
                .poll()
                .map_err(ServerError::ServiceError)?;

            match next_response {
                Async::Ready(Some(response)) => {
                    self.response_queue
                        .unbounded_send(response)
                        .map_err(|_| ServerError::ConnectionClosed)?;
                }
                Async::Ready(None) => {
                    if self.no_more_requests {
                        let _ = self.response_queue.close();
                    }
                    break;
                }
                Async::NotReady => break,
            }
        }

        Ok(())
    }

    fn poll_requests(&mut self) -> Result<(), ServerErrorAlias<S, T>> {
        loop {
            let new_request = self.incoming_requests
                .poll()
                .map_err(ServerError::ReceiveError)?;

            match new_request {
                Async::Ready(Some(request)) => {
                    self.active_requests
                        .push(self.service.call(request));
                }
                Async::Ready(None) => {
                    self.no_more_requests = true;
                    break;
                }
                Async::NotReady => break,
            }
        }

        Ok(())
    }

    fn poll_sender(&mut self) -> Poll<(), ServerErrorAlias<S, T>> {
        match self.response_sender.poll()? {
            Async::Ready(_) => Ok(Async::Ready(())),
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}

impl<S, T, H> Future for GenericServer<S, T, H>
where
    S: Service,
    T: Stream<Item = S::Request> + Sink<SinkItem = S::Response>,
    H: StreamOfFutureResults<S::Future>,
{
    type Item = ();
    type Error = ServerErrorAlias<S, T>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.poll_requests()?;
        self.poll_responses()?;
        self.poll_sender()
    }
}
