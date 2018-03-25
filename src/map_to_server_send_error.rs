use futures::{Poll, Sink, StartSend};
use std::marker::PhantomData;

use super::server_error::ServerError;

pub struct MapToServerSendError<T, E>
where
    T: Sink,
{
    sink: T,
    _error: PhantomData<E>,
}

impl<T, E> From<T> for MapToServerSendError<T, E>
where
    T: Sink,
{
    fn from(sink: T) -> Self {
        MapToServerSendError {
            sink,
            _error: PhantomData,
        }
    }
}

impl<T, I, O, S> Sink for MapToServerSendError<T, ServerError<I, O, S>>
where
    T: Sink<SinkError = O>,
{
    type SinkItem = T::SinkItem;
    type SinkError = ServerError<I, O, S>;

    fn start_send(
        &mut self,
        item: Self::SinkItem,
    ) -> StartSend<Self::SinkItem, Self::SinkError> {
        self.sink
            .start_send(item)
            .map_err(ServerError::SendError)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.sink
            .poll_complete()
            .map_err(ServerError::SendError)
    }
}
