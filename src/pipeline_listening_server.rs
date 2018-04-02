use futures::{Future, Poll, Sink, Stream, stream::FuturesOrdered};
use tokio_service::Service;

use super::generic_listening_server::{ErrorAlias, GenericListeningServer};

pub struct PipelineListeningServer<S, T>
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
        FuturesOrdered<<S::Item as Service>::Future>,
    >,
}

impl<S, T> PipelineListeningServer<S, T>
where
    S: Stream,
    S::Item: Service,
    T: Stream,
    T::Item: Sink<SinkItem = <S::Item as Service>::Response>
        + Stream<Item = <S::Item as Service>::Request>,
{
    pub fn new(services: S, transports: T) -> Self {
        PipelineListeningServer {
            listener: GenericListeningServer::new(services, transports),
        }
    }
}

impl<S, T> Future for PipelineListeningServer<S, T>
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
