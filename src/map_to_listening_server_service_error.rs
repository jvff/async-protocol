use std::marker::PhantomData;

use futures::{Poll, Sink, Stream};
use tokio_service::Service;

use super::{listening_server_error::ListeningServerError, server_error::ServerError};

pub struct MapToListeningServerServiceError<S, T>
where
    S: Stream,
    S::Item: Service,
    T: Stream,
    T::Item: Sink + Stream,
{
    services: S,
    _transports: PhantomData<T>,
}

impl<S, T> From<S> for MapToListeningServerServiceError<S, T>
where
    S: Stream,
    S::Item: Service,
    T: Stream,
    T::Item: Sink + Stream,
{
    fn from(services: S) -> Self {
        MapToListeningServerServiceError {
            services,
            _transports: PhantomData,
        }
    }
}

impl<S, T> Stream for MapToListeningServerServiceError<S, T>
where
    S: Stream,
    S::Item: Service,
    T: Stream,
    T::Item: Sink + Stream,
{
    type Item = S::Item;
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
        self.services
            .poll()
            .map_err(ListeningServerError::ServiceError)
    }
}
