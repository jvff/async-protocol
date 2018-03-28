use futures::stream::FuturesOrdered;
use futures::{Future, Poll, Sink, Stream};
use tokio_service::Service;

use super::generic_server::{GenericServer, ServerErrorAlias};

pub struct PipelineServer<S, T>
where
    S: Service,
    T: Stream<Item = S::Request> + Sink<SinkItem = S::Response>,
{
    server: GenericServer<S, T, FuturesOrdered<S::Future>>,
}

impl<S, T> PipelineServer<S, T>
where
    S: Service,
    T: Stream<Item = S::Request> + Sink<SinkItem = S::Response>,
{
    pub fn new(service: S, transport: T) -> Self {
        PipelineServer {
            server: GenericServer::new(service, transport),
        }
    }
}

impl<S, T> Future for PipelineServer<S, T>
where
    S: Service,
    T: Stream<Item = S::Request> + Sink<SinkItem = S::Response>,
{
    type Item = ();
    type Error = ServerErrorAlias<S, T>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.server.poll()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::Async;
    use futures::sync::mpsc;
    use tokio_core::reactor::{Core, Timeout};

    use super::*;
    use tests::common::{SinkStream, ToUpperService};

    #[test]
    fn simple_operation() {
        let service = ToUpperService;

        let (mut in_tx, in_rx) = mpsc::channel(2);
        let (out_tx, mut out_rx) = mpsc::channel(2);
        let transport = SinkStream::new(out_tx, in_rx);

        let server = PipelineServer::new(service, transport);

        let first_request = "first request";
        let second_request = "second request";

        let first_response = first_request.to_uppercase();
        let second_response = second_request.to_uppercase();

        in_tx
            .try_send(first_request.to_string())
            .unwrap();
        in_tx
            .try_send(second_request.to_string())
            .unwrap();
        in_tx.close().unwrap();

        let mut reactor = Core::new().unwrap();
        let timeout =
            Timeout::new(Duration::from_secs(1), &reactor.handle()).unwrap();

        assert!(reactor.run(timeout.select2(server)).is_ok());

        assert_eq!(receive(&mut out_rx), first_response);
        assert_eq!(receive(&mut out_rx), second_response);
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
