use futures::{Async, Future, Poll};

use super::client_error::ClientError;
use super::dispatcher::Dispatcher;
use super::map_to_client_receive_error::MapToClientReceiveError;
use super::receiver::Receiver;

pub struct ClientReceiver<'d, D, S>
where
    D: Dispatcher + 'd,
    S: Future,
{
    dispatcher: &'d D,
    sender: S,
}

impl<'d, D, S> ClientReceiver<'d, D, S>
where
    D: Dispatcher + 'd,
    S: Future,
{
    pub fn new(dispatcher: &'d D, sender: S) -> Self {
        ClientReceiver {
            dispatcher,
            sender,
        }
    }
}

impl<'d, D, S> Future for ClientReceiver<'d, D, S>
where
    D: Dispatcher + 'd,
    S: Future,
{
    type Item = MapToClientReceiveError<Receiver<'d, D>, D::Error, S::Error>;
    type Error = ClientError<D::Error, S::Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        try_ready!(
            self.sender
                .poll()
                .map_err(ClientError::SendError)
        );

        Ok(Async::Ready(
            self.dispatcher.spawn_receiver().into(),
        ))
    }
}
