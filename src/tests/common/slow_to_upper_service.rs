use std::io;
use std::time::Duration;

use futures::future::{Flatten, Future, FutureResult, IntoFuture, Join, Map};
use tokio_core::reactor::{Handle, Timeout};
use tokio_service::Service;

pub struct SlowToUpperService {
    handle: Handle,
}

impl SlowToUpperService {
    pub fn new(handle: Handle) -> Self {
        SlowToUpperService { handle }
    }
}

impl Service for SlowToUpperService {
    type Request = (String, Duration);
    type Response = String;
    type Error = io::Error;
    type Future = Map<
        Join<
            Flatten<FutureResult<Timeout, Self::Error>>,
            FutureResult<Self::Response, Self::Error>,
        >,
        fn(((), String)) -> Self::Response,
    >;

    fn call(&self, request: Self::Request) -> Self::Future {
        let (data, delay) = request;
        let delay_future = Timeout::new(delay, &self.handle)
            .into_future()
            .flatten();

        delay_future
            .join(Ok(data.to_uppercase()))
            .map(|result| result.1)
    }
}
