use std::io;

use telemetry_core::FRAME_SIZE_MAX;
use tokio::{io::AsyncReadExt, net::UnixStream, sync::watch};

use crate::system::IOTaskState;

// config: machines: scales_ff01: scales_s0
// for each machine: expose a stream -> recorder

// live can connect ... 

pub async fn run(
    state_tx: watch::Sender<IOTaskState>,
    mut stream: UnixStream
) -> io::Result<()> {
    let mut buf = [0u8; FRAME_SIZE_MAX];

    state_tx.send(IOTaskState::Idle).expect("rx outlives tx");

    let len = stream.read_u32().await? as usize;
    stream.read_exact(&mut buf[0..len]).await?;

    state_tx.send(IOTaskState::Processing).expect("rx outlives tx");

    /*
    Use generics V -> validator function

    let data = &buf[..len];

    if Event::decode(data).is_none() {
        // data is malformed, so we are out of sync with the stream
        anyhow::bail!("Received Malformed data");
    };
    */

    //TODO: send event to live and recorder

    Ok(())
}