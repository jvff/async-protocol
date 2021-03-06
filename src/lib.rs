extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate futures;
extern crate tokio_core;
#[cfg(feature = "tcp")]
extern crate tokio_io;
extern crate tokio_service;

mod delayed_add;
mod message_with_id;
mod ready_queue;
mod stream_of_future_results;

mod dispatcher;
mod fifo_dispatcher;
mod multiplex_dispatcher;
mod receiver;

mod request_sender;

mod client_error;
mod client_receiver;
mod map_to_client_receive_error;
mod multiplex_client;
mod pipeline_client;

mod generic_server;
mod map_to_server_send_error;
mod multiplex_server;
mod pipeline_server;
mod server_error;

mod generic_listening_server;
mod listening_server_error;
mod map_to_listening_server_server_error;
mod map_to_listening_server_service_error;
mod map_to_listening_server_transport_error;
mod multiplex_listening_server;
mod pipeline_listening_server;

#[cfg(feature = "tcp")]
mod tcp;

#[cfg(test)]
pub mod tests;

pub use message_with_id::MessageWithId;

pub use client_error::ClientError;
pub use multiplex_client::MultiplexClient;
pub use pipeline_client::PipelineClient;

pub use multiplex_server::MultiplexServer;
pub use pipeline_server::PipelineServer;
pub use server_error::ServerError;

pub use listening_server_error::ListeningServerError;
pub use multiplex_listening_server::MultiplexListeningServer;
pub use pipeline_listening_server::PipelineListeningServer;

#[cfg(feature = "tcp")]
pub use tcp::*;
