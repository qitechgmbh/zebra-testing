use std::io;
use tokio::{
    io::AsyncReadExt, net::UnixStream, select, sync::{broadcast, watch} 
};

use bytes::Bytes;
use telemetry_core::FRAME_SIZE_MAX;

pub async fn run(
    // communication
    mut kill_rx: watch::Receiver<()>,
    out_tx: broadcast::Sender<Bytes>,
    // inputs
    mut stream: UnixStream,
    validate: fn(&[u8]) -> bool,
) -> io::Result<()> {
    let mut buf = [0u8; FRAME_SIZE_MAX];

    loop {
        let len = select! {
            biased;

            // cancel safe
            _ = kill_rx.changed() => {
                return Ok(());
            },

            // not cancel safe but it's fine since
            // other event causes us to drop the stream
            len = stream.read_u32() => {
                len? as usize
            }
        };
        
        stream.read_exact(&mut buf[..len]).await?;

        let data = &buf[..len];
        if !validate(data) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData, 
                "Validation for Event failed. Stream out of sync!"
            ));
        }

        out_tx.send(Bytes::copy_from_slice(data))
            .expect("rx must outlive tx");
    }
}