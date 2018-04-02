use futures::{Future, Poll, Sink, Stream, stream::{FuturesUnordered, Zip}};
use tokio_service::Service;

use super::{
    generic_server::GenericServer,
    listening_server_error::ListeningServerError,
    map_to_listening_server_server_error::MapToListeningServerServerError,
    map_to_listening_server_service_error::MapToListeningServerServiceError,
    map_to_listening_server_transport_error::MapToListeningServerTransportError,
    server_error::ServerError, stream_of_future_results::StreamOfFutureResults,
};

pub type ErrorAlias<S: Stream, T: Stream, SI: Service, TI: Sink + Stream> =
    ListeningServerError<
        S::Error,
        T::Error,
        ServerError<TI::Error, TI::SinkError, SI::Error>,
    >;

pub struct GenericListeningServer<S, T, H>
where
    S: Stream,
    S::Item: Service,
    T: Stream,
    T::Item: Sink<SinkItem = <S::Item as Service>::Response>
        + Stream<Item = <S::Item as Service>::Request>,
    H: StreamOfFutureResults<<S::Item as Service>::Future>,
{
    active_servers: FuturesUnordered<
        MapToListeningServerServerError<
            GenericServer<S::Item, T::Item, H>,
            S,
            T,
        >,
    >,
    endpoints: Zip<
        MapToListeningServerServiceError<S, T>,
        MapToListeningServerTransportError<S, T>,
    >,
    listening: bool,
}

impl<S, T, H> GenericListeningServer<S, T, H>
where
    S: Stream,
    S::Item: Service,
    T: Stream,
    T::Item: Sink<SinkItem = <S::Item as Service>::Response>
        + Stream<Item = <S::Item as Service>::Request>,
    H: StreamOfFutureResults<<S::Item as Service>::Future>,
{
    pub fn new(services: S, transports: T) -> Self {
        let transports = MapToListeningServerTransportError::from(transports);
        let services = MapToListeningServerServiceError::from(services);

        GenericListeningServer {
            active_servers: FuturesUnordered::new(),
            endpoints: services.zip(transports),
            listening: true,
        }
    }

    fn advance_active_servers(
        &mut self,
    ) -> Poll<(), ErrorAlias<S, T, S::Item, T::Item>> {
        loop {
            try_ready!(self.active_servers.poll());
        }
    }
}

impl<S, T, H> Future for GenericListeningServer<S, T, H>
where
    S: Stream,
    S::Item: Service,
    T: Stream,
    T::Item: Sink<SinkItem = <S::Item as Service>::Response>
        + Stream<Item = <S::Item as Service>::Request>,
    H: StreamOfFutureResults<<S::Item as Service>::Future>,
{
    type Item = ();
    type Error = ErrorAlias<S, T, S::Item, T::Item>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.advance_active_servers()?;

        while self.listening {
            let endpoint = try_ready!(self.endpoints.poll());

            if let Some((service, transport)) = endpoint {
                let server = GenericServer::new(service, transport).into();

                self.active_servers.push(server);
            } else {
                self.listening = false;
            }
        }

        self.advance_active_servers()
    }
}
