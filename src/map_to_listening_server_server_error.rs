use std::marker::PhantomData;

use futures::{Future, Poll, Sink, Stream};
use tokio_service::Service;

use super::{listening_server_error::ListeningServerError, server_error::ServerError};

pub struct MapToListeningServerServerError<F, S, T>
where
    S: Stream,
    S::Item: Service,
    T: Stream,
    T::Item: Sink + Stream,
    F: Future<
        Error = ServerError<
            <T::Item as Stream>::Error,
            <T::Item as Sink>::SinkError,
            <S::Item as Service>::Error,
        >,
    >,
{
    future: F,
    _services: PhantomData<S>,
    _transports: PhantomData<T>,
}

impl<F, S, T> From<F> for MapToListeningServerServerError<F, S, T>
where
    S: Stream,
    S::Item: Service,
    T: Stream,
    T::Item: Sink + Stream,
    F: Future<
        Error = ServerError<
            <T::Item as Stream>::Error,
            <T::Item as Sink>::SinkError,
            <S::Item as Service>::Error,
        >,
    >,
{
    fn from(future: F) -> Self {
        MapToListeningServerServerError {
            future,
            _services: PhantomData,
            _transports: PhantomData,
        }
    }
}

impl<F, S, T> Future for MapToListeningServerServerError<F, S, T>
where
    S: Stream,
    S::Item: Service,
    T: Stream,
    T::Item: Sink + Stream,
    F: Future<
        Error = ServerError<
            <T::Item as Stream>::Error,
            <T::Item as Sink>::SinkError,
            <S::Item as Service>::Error,
        >,
    >,
{
    type Item = F::Item;
    type Error = ListeningServerError<S::Error, T::Error, F::Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.future
            .poll()
            .map_err(ListeningServerError::ServerError)
    }
}
