use futures::future::{FutureResult, IntoFuture};
use tokio_service::Service;

pub struct ToUpperService;

impl Service for ToUpperService {
    type Request = String;
    type Response = String;
    type Error = ();
    type Future = FutureResult<Self::Response, Self::Error>;

    fn call(&self, request: Self::Request) -> Self::Future {
        Ok(request.to_uppercase()).into_future()
    }
}
