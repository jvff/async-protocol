use std::io;
use std::net::SocketAddr;

use futures::{Future, Poll, Stream};
use tokio_core::{net::TcpListener, reactor::Handle};
use tokio_io::codec::{Decoder, Encoder};
use tokio_service::Service;

use super::{
    incoming_transports::IncomingTransports,
    super::{
        generic_listening_server::GenericListeningServer,
        listening_server_error::ListeningServerError,
        server_error::ServerError,
        stream_of_future_results::StreamOfFutureResults,
    },
};

pub type ErrorAlias<S: Stream, SI: Service, C> = ListeningServerError<
    S::Error,
    io::Error,
    ServerError<<C as Decoder>::Error, <C as Encoder>::Error, SI::Error>,
>;

pub struct GenericTcpListenerServer<S, C, H>
where
    S: Stream,
    S::Item: Service<
        Request = <C as Decoder>::Item,
        Response = <C as Encoder>::Item,
    >,
    C: Clone + Decoder + Encoder,
    H: StreamOfFutureResults<<S::Item as Service>::Future>,
{
    server: GenericListeningServer<S, IncomingTransports<C>, H>,
}

impl<S, C, H> GenericTcpListenerServer<S, C, H>
where
    S: Stream,
    S::Item: Service<
        Request = <C as Decoder>::Item,
        Response = <C as Encoder>::Item,
    >,
    C: Clone + Decoder + Encoder,
    H: StreamOfFutureResults<<S::Item as Service>::Future>,
{
    pub fn listen(
        services: S,
        address: &SocketAddr,
        codec: C,
        handle: &Handle,
    ) -> io::Result<Self> {
        let listener = TcpListener::bind(address, handle)?;
        let incoming = listener.incoming();
        let transports = IncomingTransports::new(codec, incoming);

        Ok(GenericTcpListenerServer {
            server: GenericListeningServer::new(services, transports),
        })
    }
}

impl<S, C, H> Future for GenericTcpListenerServer<S, C, H>
where
    S: Stream,
    S::Item: Service<
        Request = <C as Decoder>::Item,
        Response = <C as Encoder>::Item,
    >,
    C: Clone + Decoder + Encoder,
    H: StreamOfFutureResults<<S::Item as Service>::Future>,
{
    type Item = ();
    type Error = ErrorAlias<S, S::Item, C>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.server.poll()
    }
}
