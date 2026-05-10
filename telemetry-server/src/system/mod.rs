use std::{io, sync::Arc, time::Duration};

use tokio::{net::{TcpListener, TcpStream, UnixStream}, select, signal, spawn, sync::mpsc::{self, Sender}, task::JoinHandle, time::sleep};

use crate::config::Config;



pub enum SystemAction {
    Shutdown,
    SpawnIngest(UnixStream),
    SpawnRouter(TcpStream),
    SpawnLive(TcpStream),
    SpawnQuery(TcpStream), // -> query
}

// EventIngestor

pub async fn run(config: Arc<Config>) -> io::Result<()> {
    // let unix_task = supervise(move || {
    //     let config = config.clone();
    //     run_tcp_listener(config)
    // });

    let (tx, mut rx) = mpsc::channel::<SystemAction>(128);

    // start task to receive sigterm
    start_signal_handler(tx.clone());

    loop {
        let action = rx.recv().await.expect("tx may not be dropped");

        match action {
            SystemAction::Shutdown => break,

            SystemAction::SpawnIngest(stream) => {

            }

            SystemAction::SpawnLive(stream) => {

            }

            SystemAction::SpawnQuery(stream) => {

            }
        }
    }

    // stop listeners
    // unix_task.abort();

    // send recorders kill action

    // stop ingest once it exits pending stage

    Ok(())
}

fn start_signal_handler(tx: Sender<SystemAction>) {
    use signal::unix::signal;
    use signal::unix::SignalKind;

    spawn(async move {
        let mut sigterm = signal(SignalKind::terminate())
            .expect("failed to install SIGTERM handler");

        let mut sigint = signal(SignalKind::interrupt())
            .expect("failed to install SIGINT handler");

        select! {
            _ = sigterm.recv() => {
                let _ = tx.send(SystemAction::Shutdown);
            }
            _ = sigint.recv() => {
                let _ = tx.send(SystemAction::Shutdown);
            }
        }
    });
}

pub fn supervise<F, Fut, Launcher>(launcher: Launcher) -> JoinHandle<()>
where
    Launcher: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = io::Result<()>> + Send + 'static,
{
    spawn(async move {
        loop {
            let handle = spawn(launcher());

            let result = handle.await;

            match result {
                Ok(Ok(())) => {
                    println!("Worker exited normally, restarting...");
                }
                Ok(Err(e)) => {
                    println!("Worker error: {e}, restarting...");
                }
                Err(e) => {
                    println!("Worker panicked: {e}, restarting...");
                }
            }

            sleep(Duration::from_millis(200)).await;
        }
    })
}

pub async fn run_tcp_listener(config: Arc<Config>) -> io::Result<()> {
    let listener = TcpListener::bind((config.tcp_ipv4.as_str(), config.tcp_port)).await?;

    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(e) => {
                eprintln!("Failed to Accept Unix connection: {e}");
                continue;
            },
        };

        // TODO: send stream to System/Router
    }
}