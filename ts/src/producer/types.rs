use tokio::{net::UnixStream};

pub type CompleteReason = crate::types::CompleteReason<CompleteReasonCustom>;
pub type Result = std::result::Result<StateTransition, CompleteReason>;
pub type Stream = crate::stream::Stream<UnixStream>;

#[derive(Debug, Clone, Copy)]
pub enum State {
    RecvPort,
    RecvData,
    SendRefuse,
    SendExit,
}

#[derive(Debug)]
pub enum NextState {
    RecvData(String),
}

#[derive(Debug)]
pub enum RefuseReason {
    NoSuchPort,
    Occupied,
}

#[derive(Debug)]
pub enum InvalidDataError {
    NoSuchPort,
    Occupied,
}

#[derive(Debug)]
pub enum CompleteReasonCustom {
    Refused(RefuseReason),
    InvalidData,
}

pub struct StateTransition {
    pub next:   NextState,
    pub stream: Stream,
}