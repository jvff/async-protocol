use std::marker::PhantomData;

use futures::{Future, Poll};

use super::client_error::ClientError;

pub struct MapToClientReceiveError<F, I, O>
where
    F: Future<Error = I>,
{
    future: F,
    _receive_error: PhantomData<I>,
    _send_error: PhantomData<O>,
}

impl<F, I, O> From<F> for MapToClientReceiveError<F, I, O>
where
    F: Future<Error = I>,
{
    fn from(future: F) -> Self {
        MapToClientReceiveError {
            future,
            _receive_error: PhantomData,
            _send_error: PhantomData,
        }
    }
}

impl<F, I, O> Future for MapToClientReceiveError<F, I, O>
where
    F: Future<Error = I>,
{
    type Item =  F::Item;
    type Error = ClientError<I, O>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.future.poll().map_err(ClientError::ReceiveError)
    }
}
