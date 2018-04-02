use std::{io, mem, net::SocketAddr};

use futures::{Async, AsyncSink, Future, Poll, Sink, StartSend, Stream};
use tokio_core::{net::{TcpStream, TcpStreamNew}, reactor::Handle};
use tokio_io::{AsyncRead, codec::{Decoder, Encoder, Framed}};

pub enum TcpClientTransport<C>
where
    C: Decoder + Encoder,
{
    Connecting(TcpStreamNew, C),
    Connected(Framed<TcpStream, C>),
    Polling,
}

impl<C> TcpClientTransport<C>
where
    C: Decoder + Encoder,
{
    pub fn connect(address: &SocketAddr, codec: C, handle: &Handle) -> Self {
        TcpClientTransport::Connecting(
            TcpStream::connect(address, handle),
            codec,
        )
    }

    fn is_connecting(&self) -> bool {
        match *self {
            TcpClientTransport::Connecting(..) => true,
            _ => false,
        }
    }

    fn wait_for_connection(&mut self) -> Poll<(), io::Error> {
        use self::TcpClientTransport::*;

        if self.is_connecting() {
            let old_state = mem::replace(self, TcpClientTransport::Polling);

            if let Connecting(mut connecting, codec) = old_state {
                match connecting.poll() {
                    Ok(Async::Ready(connected)) => {
                        let transport = connected.framed(codec);

                        mem::replace(self, Connected(transport));

                        Ok(Async::Ready(()))
                    }
                    Ok(Async::NotReady) => {
                        mem::replace(self, Connecting(connecting, codec));

                        Ok(Async::NotReady)
                    }
                    Err(error) => {
                        mem::replace(self, Connecting(connecting, codec));

                        Err(error)
                    }
                }
            } else {
                unreachable!("State was tested to be Connecting");
            }
        } else {
            Ok(Async::Ready(()))
        }
    }
}

impl<C> Stream for TcpClientTransport<C>
where
    C: Decoder + Encoder,
{
    type Item = <C as Decoder>::Item;
    type Error = <C as Decoder>::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        try_ready!(self.wait_for_connection());

        if let TcpClientTransport::Connected(ref mut transport) = *self {
            transport.poll()
        } else {
            unreachable!(
                "Function wait_for_connection left TcpClientTransport in \
                 invalid state"
            );
        }
    }
}

impl<C> Sink for TcpClientTransport<C>
where
    C: Decoder + Encoder,
{
    type SinkItem = <C as Encoder>::Item;
    type SinkError = <C as Encoder>::Error;

    fn start_send(
        &mut self,
        item: Self::SinkItem,
    ) -> StartSend<Self::SinkItem, Self::SinkError> {
        if let Async::NotReady = self.wait_for_connection()? {
            return Ok(AsyncSink::NotReady(item));
        }

        if let TcpClientTransport::Connected(ref mut transport) = *self {
            transport.start_send(item)
        } else {
            unreachable!(
                "Function wait_for_connection left TcpClientTransport in \
                 invalid state"
            );
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        try_ready!(self.wait_for_connection());

        if let TcpClientTransport::Connected(ref mut transport) = *self {
            transport.poll_complete()
        } else {
            unreachable!(
                "Function wait_for_connection left TcpClientTransport in \
                 invalid state"
            );
        }
    }
}
