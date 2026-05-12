use std::{io::{self, ErrorKind}, sync::Arc};
use tokio::{fs, net::UnixListener, select, spawn, sync::{mpsc, watch}, task::JoinHandle};

use crate::{config::Config, types::SystemEvent};

pub struct Listener {
    kill_tx: watch::Sender<()>,
    handle:  JoinHandle<()>,
}

impl Listener {
    pub async fn new(
        config: Arc<Config>,
        sys_tx: mpsc::Sender<SystemEvent>, 
    ) -> io::Result<Self> {
        if let Err(e) = fs::remove_file(&config.sock_path).await {
            if e.kind() != ErrorKind::NotFound {
                return Err(e);
            }
        }

        let listener = UnixListener::bind(&config.sock_path)?;
        let (kill_tx, kill_rx) = watch::channel(());

        let handle = spawn(Self::run(listener, sys_tx, kill_rx));

        Ok(Self { kill_tx, handle })
    }

    pub async fn stop(self) {
        let res = self.kill_tx.send(());
        if res.is_err() {
            // tx can only be dropped if run() died
            assert!(self.handle.is_finished());
        }
        
        if let Err(e) = self.handle.await {
            eprintln!("Error while shutting down: {e}");
        }
    }

    async fn run(
        listener:    UnixListener,
        sys_tx:      mpsc::Sender<SystemEvent>,
        mut kill_rx: watch::Receiver<()>
    ) {
        println!("Listening for Unix connections on socket {:?}", listener.local_addr());

        loop {
            let result = select! {
                biased;

                _ = kill_rx.changed() => {
                    eprintln!("Received Shutdown signal");
                    break;
                }

                v = listener.accept() => { v }
            };

            let stream = match result {
                Ok((stream, _)) => stream,
                Err(e) => {
                    eprintln!("Failed to Accept Unix connection: {e}");
                    continue;
                },
            };

            let event = SystemEvent::ProducerAccepted(stream);
            sys_tx.send(event).await.expect("rx outlives tx");
        }
    }
}