#[derive(Debug, Fail)]
pub enum ServerError<I, O, S> {
    #[fail(display = "failed to send a response because the connection was \
                      closed")]
    ConnectionClosed,

    #[fail(display = "failed to receive request: {}", _0)]
    ReceiveError(#[cause] I),

    #[fail(display = "failed to service request: {}", _0)]
    ServiceError(#[cause] S),

    #[fail(display = "failed to send response: {}", _0)]
    SendError(#[cause] O),
}

impl<I, O, S> From<()> for ServerError<I, O, S> {
    fn from(_: ()) -> Self {
        unreachable!("UnboundedReceiver used to queue responses failed");
    }
}
