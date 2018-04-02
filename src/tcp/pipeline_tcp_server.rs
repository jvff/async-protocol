use futures::stream::FuturesOrdered;
use futures::{Future, Poll};
use tokio_core::net::TcpStream;
use tokio_io::codec::{Decoder, Encoder};
use tokio_service::Service;

use super::generic_tcp_server::{GenericTcpServer, ServerErrorAlias};

pub struct PipelineTcpServer<S, C>
where
    S: Service,
    C: Decoder<Item = S::Request> + Encoder<Item = S::Response>,
{
    server: GenericTcpServer<S, C, FuturesOrdered<S::Future>>,
}

impl<S, C> PipelineTcpServer<S, C>
where
    S: Service,
    C: Decoder<Item = S::Request> + Encoder<Item = S::Response>,
{
    pub fn new(service: S, connection: TcpStream, codec: C) -> Self {
        PipelineTcpServer {
            server: GenericTcpServer::new(service, connection, codec),
        }
    }
}

impl<S, C> Future for PipelineTcpServer<S, C>
where
    S: Service,
    C: Decoder<Item = S::Request> + Encoder<Item = S::Response>,
{
    type Item = ();
    type Error = ServerErrorAlias<S, C>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.server.poll()
    }
}
