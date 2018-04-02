use std::io;

use futures::{Async, Poll, Stream};
use tokio_core::net::{Incoming, TcpStream};
use tokio_io::{AsyncRead, codec::{Decoder, Encoder, Framed}};

pub struct IncomingTransports<C>
where
    C: Clone + Decoder + Encoder,
{
    codec: C,
    connections: Incoming,
}

impl<C> IncomingTransports<C>
where
    C: Clone + Decoder + Encoder,
{
    pub fn new(codec: C, connections: Incoming) -> Self {
        IncomingTransports {
            codec,
            connections,
        }
    }
}

impl<C> Stream for IncomingTransports<C>
where
    C: Clone + Decoder + Encoder,
{
    type Item = Framed<TcpStream, C>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let connection = try_ready!(self.connections.poll());
        let transport = connection.map(|(connection, _address)| {
            connection.framed(self.codec.clone())
        });

        Ok(Async::Ready(transport))
    }
}
