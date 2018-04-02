#[derive(Debug, Fail)]
pub enum ListeningServerError<S, T, A> {
    #[fail(display = "failed to recive a new transport: {}", _0)]
    TransportError(#[cause] T),

    #[fail(display = "failed to receive request: {}", _0)]
    ServiceError(#[cause] S),

    #[fail(display = "one of the active servers failed: {}", _0)]
    ServerError(#[cause] A),
}
