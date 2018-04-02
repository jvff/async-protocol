use std::marker::PhantomData;

use futures::{Poll, Sink, Stream};
use tokio_service::Service;

use super::{listening_server_error::ListeningServerError, server_error::ServerError};

pub struct MapToListeningServerTransportError<S, T>
where
    S: Stream,
    S::Item: Service,
    T: Stream,
    T::Item: Sink + Stream,
{
    _services: PhantomData<S>,
    transports: T,
}

impl<S, T> From<T> for MapToListeningServerTransportError<S, T>
where
    S: Stream,
    S::Item: Service,
    T: Stream,
    T::Item: Sink + Stream,
{
    fn from(transports: T) -> Self {
        MapToListeningServerTransportError {
            _services: PhantomData,
            transports,
        }
    }
}

impl<S, T> Stream for MapToListeningServerTransportError<S, T>
where
    S: Stream,
    S::Item: Service,
    T: Stream,
    T::Item: Sink + Stream,
{
    type Item = T::Item;
    type Error = ListeningServerError<
        S::Error,
        T::Error,
        ServerError<
            <T::Item as Stream>::Error,
            <T::Item as Sink>::SinkError,
            <S::Item as Service>::Error,
        >,
    >;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.transports
            .poll()
            .map_err(ListeningServerError::TransportError)
    }
}
