use tokio::{spawn, sync::mpsc};
use crate::types::SystemEvent;
use super::{Result, CompleteReason};

pub async fn execute<F>(
    sys_tx: mpsc::Sender<SystemEvent>,
    id: u64,
    f: F,
)
where
    F: Future<Output = Result> + Send + 'static,
{
    let handle = spawn(f).await;

    let event = match handle {
        Ok(result) => {
            match result {
                Ok(state_change) => {
                    SystemEvent::ProducerStateChanged(id, state_change)
                }

                Err(completed_reason) => {
                    SystemEvent::ProducerCompleted(id, completed_reason)
                }
            }
        }

        Err(e) => {
            SystemEvent::ProducerCompleted(id, CompleteReason::JoinFailure(e))
        }
    };

    sys_tx.send(event).await.expect("rx outlives tx");
}