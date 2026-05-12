use std::{fmt::Debug, io};

use tokio::{net::UnixStream, task::JoinError};
use crate::producer;

pub enum SystemEvent {
    // producer events
    ProducerAccepted(UnixStream),
    ProducerStateChanged(u64, producer::StateTransition),
    ProducerCompleted(u64, producer::CompleteReason),

    // Recorder events
    RecorderAborted(&'static str),
    QueueFlush(),

    // consumer events
    // ConsumerRedirect(http::RedirectRequest),
    // ConsumerTerminated(TerminationReason),
}

#[derive(Debug)]
pub enum CompleteReason<Custom: Debug> {
    Custom(Custom),
    Timeout,
    Shutdown,
    ClientDisconnected,
    IoFailure(io::Error),
    JoinFailure(JoinError),
}

#[derive(Debug)]
pub enum ClientExitReason {
    Timeout,
    Shutdown,
    Disconnected,
    IoFailure(io::Error),
    JoinFailure(JoinError),
}

pub type ClientTaskResult<T> = std::result::Result<T, ClientExitReason>;
pub type ClientTask<T> = tokio::task::JoinHandle<ClientTaskResult<T>>;