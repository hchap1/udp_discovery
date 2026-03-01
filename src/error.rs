#[derive(Debug, Clone)]
pub enum Error {
    NoEndpoint,
    BindFailed,
    BroadcastFailed,
    HandleDropped,
    RecvFailed,
    InvalidIdentifier
}
