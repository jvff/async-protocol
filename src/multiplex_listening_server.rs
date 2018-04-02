use futures::{Future, Poll, Sink, Stream, stream::FuturesUnordered};
use tokio_service::Service;

use super::generic_listening_server::{ErrorAlias, GenericListeningServer};

pub struct MultiplexListeningServer<S, T>
where
    S: Stream,
    S::Item: Service,
    T: Stream,
    T::Item: Sink<SinkItem = <S::Item as Service>::Response>
        + Stream<Item = <S::Item as Service>::Request>,
{
    listener: GenericListeningServer<
        S,
        T,
        FuturesUnordered<<S::Item as Service>::Future>,
    >,
}

impl<S, T> MultiplexListeningServer<S, T>
where
    S: Stream,
    S::Item: Service,
    T: Stream,
    T::Item: Sink<SinkItem = <S::Item as Service>::Response>
        + Stream<Item = <S::Item as Service>::Request>,
{
    pub fn new(services: S, transports: T) -> Self {
        MultiplexListeningServer {
            listener: GenericListeningServer::new(services, transports),
        }
    }
}

impl<S, T> Future for MultiplexListeningServer<S, T>
where
    S: Stream,
    S::Item: Service,
    T: Stream,
    T::Item: Sink<SinkItem = <S::Item as Service>::Response>
        + Stream<Item = <S::Item as Service>::Request>,
{
    type Item = ();
    type Error = ErrorAlias<S, T, S::Item, T::Item>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.listener.poll()
    }
}
