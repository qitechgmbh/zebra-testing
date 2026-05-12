use std::time::Duration;
use tokio::sync::broadcast;
use bytes::Bytes;

use crate::stream::ExitCondition;
use crate::types::CompleteReason;
use super::types::{Stream, Result, CompleteReasonCustom};

pub async fn run(
    mut stream: Stream,
    validate: fn(&[u8]) -> bool,
    out_tx: broadcast::Sender<Bytes>,
) -> Result {
    let mut buf = [0u8; 512];

    loop {
        let len = stream.read_u32(ExitCondition::Shutdown).await? as usize;

        stream.read_exact(
            &mut buf[..len], 
            ExitCondition::Timer(Duration::from_secs(2))
        ).await?;

        let data = &buf[..len];

        if !validate(data) {
            let reason = CompleteReasonCustom::InvalidData;
            return Err(CompleteReason::Custom(reason));
        }

        out_tx.send(Bytes::copy_from_slice(data))
            .expect("rx must outlive tx");
    }
}