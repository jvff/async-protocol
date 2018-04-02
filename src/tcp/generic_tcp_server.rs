use futures::{Future, Poll};
use tokio_core::net::TcpStream;
use tokio_io::{AsyncRead, codec::{Decoder, Encoder, Framed}};
use tokio_service::Service;

use super::super::{
    generic_server::{GenericServer, ServerErrorAlias as GenericServerError},
    stream_of_future_results::StreamOfFutureResults,
};

pub type ServerErrorAlias<S: Service, C> =
    GenericServerError<S, Framed<TcpStream, C>>;

pub struct GenericTcpServer<S, C, H>
where
    S: Service,
    C: Decoder<Item = S::Request> + Encoder<Item = S::Response>,
    H: StreamOfFutureResults<S::Future>,
{
    server: GenericServer<S, Framed<TcpStream, C>, H>,
}

impl<S, C, H> GenericTcpServer<S, C, H>
where
    S: Service,
    C: Decoder<Item = S::Request> + Encoder<Item = S::Response>,
    H: StreamOfFutureResults<S::Future>,
{
    pub fn new(service: S, connection: TcpStream, codec: C) -> Self {
        let transport = connection.framed(codec);

        GenericTcpServer {
            server: GenericServer::new(service, transport),
        }
    }
}

impl<S, C, H> Future for GenericTcpServer<S, C, H>
where
    S: Service,
    C: Decoder<Item = S::Request> + Encoder<Item = S::Response>,
    H: StreamOfFutureResults<S::Future>,
{
    type Item = ();
    type Error = ServerErrorAlias<S, C>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.server.poll()
    }
}
