use std::io;
use std::net::SocketAddr;

use futures::{Future, Poll, Stream, stream::FuturesOrdered};
use tokio_core::reactor::Handle;
use tokio_io::codec::{Decoder, Encoder};
use tokio_service::Service;

use super::generic_tcp_listener_server::{ErrorAlias, GenericTcpListenerServer};

pub struct PipelineTcpListenerServer<S, C>
where
    S: Stream,
    S::Item: Service<
        Request = <C as Decoder>::Item,
        Response = <C as Encoder>::Item,
    >,
    C: Clone + Decoder + Encoder,
{
    listener: GenericTcpListenerServer<
        S,
        C,
        FuturesOrdered<<S::Item as Service>::Future>,
    >,
}

impl<S, C> PipelineTcpListenerServer<S, C>
where
    S: Stream,
    S::Item: Service<
        Request = <C as Decoder>::Item,
        Response = <C as Encoder>::Item,
    >,
    C: Clone + Decoder + Encoder,
{
    pub fn listen(
        services: S,
        address: &SocketAddr,
        codec: C,
        handle: &Handle,
    ) -> io::Result<Self> {
        let listener =
            GenericTcpListenerServer::listen(services, address, codec, handle)?;

        Ok(PipelineTcpListenerServer { listener })
    }
}

impl<S, C> Future for PipelineTcpListenerServer<S, C>
where
    S: Stream,
    S::Item: Service<
        Request = <C as Decoder>::Item,
        Response = <C as Encoder>::Item,
    >,
    C: Clone + Decoder + Encoder,
{
    type Item = ();
    type Error = ErrorAlias<S, S::Item, C>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.listener.poll()
    }
}
