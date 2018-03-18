#[derive(Debug, Fail)]
pub enum ClientError<I, O> {
    #[fail(display = "failed to receive response: {}", _0)]
    ReceiveError(#[cause] I),

    #[fail(display = "failed to send request: {}", _0)]
    SendError(#[cause] O),
}
