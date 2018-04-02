use std::net::SocketAddr;

use tokio_core::{net::TcpStream, reactor::Handle};
use tokio_io::codec::{Decoder, Encoder};
use tokio_service::Service;

use super::{
    super::{client_error::ClientError, pipeline_client::PipelineClient},
    tcp_client_transport::TcpClientTransport,
};

pub struct PipelineTcpClient<C>
where
    C: Decoder + Encoder,
{
    client: PipelineClient<TcpClientTransport<C>>,
}

impl<C> PipelineTcpClient<C>
where
    C: Decoder + Encoder,
{
    pub fn connect(address: &SocketAddr, codec: C, handle: &Handle) -> Self {
        let transport = TcpClientTransport::connect(address, codec, handle);

        PipelineTcpClient {
            client: PipelineClient::new(transport),
        }
    }

    pub fn with_connection(connection: TcpStream, codec: C) -> Self {
        let transport = TcpClientTransport::with_connection(connection, codec);

        PipelineTcpClient {
            client: PipelineClient::new(transport),
        }
    }
}

impl<C> Service for PipelineTcpClient<C>
where
    C: Decoder + Encoder,
{
    type Request = <C as Encoder>::Item;
    type Response = <C as Decoder>::Item;
    type Error = ClientError<<C as Decoder>::Error, <C as Encoder>::Error>;
    type Future = <PipelineClient<TcpClientTransport<C>> as Service>::Future;

    fn call(&self, request: Self::Request) -> Self::Future {
        let client = &self.client;

        client.call(request)
    }
}
