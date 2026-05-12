use std::{collections::HashMap, sync::Arc};

use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    sync::mpsc,
    net::UnixStream,
};

use crate::{config::Config, root::SystemEvent};

pub async fn run(
    sys_tx: mpsc::Sender<SystemEvent>,
    config: Arc<Config>,
    mut stream: UnixStream,
) -> io::Result<()> {
    let mut reader = BufReader::new(&mut stream);
    let mut buf    = String::new();

    let n = reader.read_line(&mut buf).await?;
    if n == 0 {
        // connection closed
        return Ok(());
    }

    let name = buf.trim_end().to_string();

    if !config.machines.contains_key(&name) {
        sys_tx.send(SystemEvent::UnixCloseWithMessage("No such machine".into(), stream))
            .await
            .expect("rx shares lifetime with tx");

        return Ok(());
    }

    sys_tx.send(SystemEvent::StartIngest(name, stream))
        .await
        .expect("rx shares lifetime with tx");

    Ok(())
}