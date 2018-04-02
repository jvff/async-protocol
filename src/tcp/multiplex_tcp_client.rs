use std::{hash::Hash, net::SocketAddr};

use tokio_core::{net::TcpStream, reactor::Handle};
use tokio_io::codec::{Decoder, Encoder};
use tokio_service::Service;

use super::{
    super::{
        client_error::ClientError, message_with_id::MessageWithId,
        multiplex_client::MultiplexClient,
    },
    tcp_client_transport::TcpClientTransport,
};

pub struct MultiplexTcpClient<C>
where
    C: Decoder + Encoder,
    <C as Decoder>::Item: MessageWithId,
    <C as Encoder>::Item:
        MessageWithId<Id = <<C as Decoder>::Item as MessageWithId>::Id>,
    <<C as Decoder>::Item as MessageWithId>::Id: Eq + Hash,
{
    client: MultiplexClient<TcpClientTransport<C>>,
}

impl<C> MultiplexTcpClient<C>
where
    C: Decoder + Encoder,
    <C as Decoder>::Item: MessageWithId,
    <C as Encoder>::Item:
        MessageWithId<Id = <<C as Decoder>::Item as MessageWithId>::Id>,
    <<C as Decoder>::Item as MessageWithId>::Id: Eq + Hash,
{
    pub fn connect(address: &SocketAddr, codec: C, handle: &Handle) -> Self {
        let transport = TcpClientTransport::connect(address, codec, handle);

        MultiplexTcpClient {
            client: MultiplexClient::new(transport),
        }
    }

    pub fn with_connection(connection: TcpStream, codec: C) -> Self {
        let transport = TcpClientTransport::with_connection(connection, codec);

        MultiplexTcpClient {
            client: MultiplexClient::new(transport),
        }
    }
}

impl<C> Service for MultiplexTcpClient<C>
where
    C: Decoder + Encoder,
    <C as Decoder>::Item: MessageWithId,
    <C as Encoder>::Item:
        MessageWithId<Id = <<C as Decoder>::Item as MessageWithId>::Id>,
    <<C as Decoder>::Item as MessageWithId>::Id: Eq + Hash,
{
    type Request = <C as Encoder>::Item;
    type Response = <C as Decoder>::Item;
    type Error = ClientError<<C as Decoder>::Error, <C as Encoder>::Error>;
    type Future = <MultiplexClient<TcpClientTransport<C>> as Service>::Future;

    fn call(&self, request: Self::Request) -> Self::Future {
        let client = &self.client;

        client.call(request)
    }
}
