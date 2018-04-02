use std::sync::Arc;

use futures::{Async, Future, Poll};

use super::client_error::ClientError;
use super::dispatcher::Dispatcher;
use super::map_to_client_receive_error::MapToClientReceiveError;
use super::receiver::Receiver;

pub struct ClientReceiver<D, S>
where
    D: Dispatcher,
    S: Future<Item = D::Seed>,
{
    dispatcher: Arc<D>,
    sender: S,
}

impl<D, S> ClientReceiver<D, S>
where
    D: Dispatcher,
    S: Future<Item = D::Seed>,
{
    pub fn new(dispatcher: Arc<D>, sender: S) -> Self {
        ClientReceiver {
            dispatcher,
            sender,
        }
    }
}

impl<D, S> Future for ClientReceiver<D, S>
where
    D: Dispatcher,
    S: Future<Item = D::Seed>,
{
    type Item = MapToClientReceiveError<Receiver<D>, D::Error, S::Error>;
    type Error = ClientError<D::Error, S::Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let seed = try_ready!(
            self.sender
                .poll()
                .map_err(ClientError::SendError)
        );

        Ok(Async::Ready(
            Dispatcher::spawn_receiver(self.dispatcher.clone(), seed).into(),
        ))
    }
}
