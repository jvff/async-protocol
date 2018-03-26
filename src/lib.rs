extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate futures;
extern crate tokio_core;
extern crate tokio_service;

mod delayed_add;
mod ready_queue;

mod fifo_dispatcher;
mod dispatcher;
mod receiver;

mod request_sender;

mod client_error;
mod client_receiver;
mod map_to_client_receive_error;
mod pipeline_client;

mod map_to_server_send_error;
mod pipeline_server;
mod server_error;

#[cfg(test)]
pub mod tests;

pub use client_error::ClientError;
pub use pipeline_client::PipelineClient;

pub use pipeline_server::PipelineServer;
pub use server_error::ServerError;
