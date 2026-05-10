use std::{mem, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration};

use telemetry_core::{Event, FRAME_SIZE_MAX};
use tokio::{io::{self, AsyncReadExt}, net::{UnixListener, UnixStream}, sync::mpsc::{Receiver, Sender}, task::{self, JoinHandle, JoinSet}, time::{sleep, timeout}};

use crate::{PayloadSender, overseer::Task};




pub struct IngestService {
    pub listener: JoinHandle<()>,
    pub tasks:    Task,
}

impl IngestService {
    pub async fn new(tx: Sender<Arc<Vec<u8>>>, port: u16) {
        
    }

    pub async fn shutdown(self) {
        self.listener.abort();
        self.client.shutdown().await;
    }
}

async fn hub() {
    
}

// IngestService

async fn run_ingest(mut shutdown_rx: Receiver<()>, socket_path: &String) {
    let listener = UnixListener::bind(socket_path).expect("idk");

    let handle = task::spawn(async move {
        loop {
            let stream = listener.accept().await.expect("idk");
            //TODO: -> start new task
        }
    });

    // wait for shutdown signal
    _ = shutdown_rx.recv().await;

    // shutdown all
    handle.abort();
}

async fn run_listener(listener: UnixListener) -> anyhow::Result<()> {
    
    // -> let stream = listener.accept().await?;
    loop {
        let stream = listener.accept().await?;
        //TODO: -> start new task
    }

    Ok(())
}

async fn cycle(stream: &mut UnixStream) -> anyhow::Result<bool> {
    let mut buf = [0u8; FRAME_SIZE_MAX];

    let len = stream.read_u32().await? as usize;
    stream.read_exact(&mut buf[0..len]).await?;

    let data = &buf[..len];

    if Event::decode(data).is_none() {
        // data is malformed, so we are out of sync with the stream
        anyhow::bail!("Received Malformed data");
    };

    println!("Received Event");

    Ok(false)
}