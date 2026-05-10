use std::{ collections::HashMap, io, sync::Arc };

use bytes::Bytes;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream, UnixListener, UnixStream}, select, signal, spawn, sync::{broadcast, mpsc::{self, Sender}, watch}, task::JoinHandle 
};

use crate::{config::Config, types::MachineDataSchema};


// ingest broadcasts message, recorder saves data into WAL, finally tell system to 
// write data into database. All it does it the system is: hey please commit 
// new batch into database. When recorder is started it checks for existing wal -> 
// notify system to write the batch

// all database operations through system. Recorder is essential for storing data

#[derive(Debug)]
pub enum SystemAction {
    SpawnUnixRouter(UnixStream),
    SpawnUnixIngest(UnixStream, String),
    SpawnTcpRouter(TcpStream),

    // endpoint, 
    WriteBatch(String, ),

    // terminates all tasks that use that database
    // closes all connections to the database
    // then try to open the database again -> init tables
    // then spawn the recorder
    // ReloadDatabase(),

    // SpawnRecorder(),
    // SpawnIngest(UnixStream),
    // SpawnRouter(TcpStream),
    // SpawnLive(TcpStream),
    // SpawnQuery(TcpStream),
}

pub struct System {
    registry: HashMap<String, RegistryEntry>,
}

// combine ingest + recording into one ?
// so we would only have to push data to live listeners

// ingest depends on db then

pub struct RegistryEntry {
    schema:   MachineDataSchema,
    event_tx: broadcast::Sender<Bytes>,
    event_rx: broadcast::Receiver<Bytes>,
    ingest:   u32,
    recorder: u32,
}

pub async fn run(config: Arc<Config>) -> io::Result<()> {
    use signal::unix::signal;
    use signal::unix::SignalKind;

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint  = signal(SignalKind::interrupt())?;

    // system channels
    let (tx, mut rx) = mpsc::channel::<SystemAction>(128);

    // Deploy termination signal hooks
    start_signal_handler(tx.clone())?;
    println!("Signal Hooks engaged");

    let unix_listener = spawn(run_unix_listener(config.clone(), tx.clone()));
    let tcp_listener  = spawn(run_tcp_listener(config.clone(), tx.clone()));

    let (router_shutdown_tx, router_shutdown_rx) = watch::channel(());
    let mut routers = Vec::<JoinHandle<io::Result<()>>>::new();
    // router.push(value);

    // start event loop
    loop {
        let action = select! {
            biased;

            _ = sigterm.recv() => { break; }
            _ = sigint.recv()  => { break; }
            action = rx.recv() => {
                action.expect("tx may not be dropped")
            }
        };

        println!("Received action: {action:?}");

        match action {
            SystemAction::SpawnUnixRouter(stream) => {
                let task   = run_unix_router(router_shutdown_rx.clone(), stream);
                let handle = spawn(task);
                routers.push(handle);
            }

            SystemAction::SpawnUnixIngest(stream, endpoint) => {

            }

            SystemAction::SpawnTcpRouter(stream) => {

            }

            SystemAction::WriteBatch(_) => {

            }
        }
    }

    println!("System shutdown started");

    println!("Stopping unix listener");
    unix_listener.abort();

    println!("Stopping tcp listener");
    tcp_listener.abort();

    println!("Terminating Routers");
    router_shutdown_tx.send(()).expect("rx shares lifetime with tx");

    println!("System shutdown complete");

    Ok(())
}

fn start_signal_handler(tx: Sender<SystemAction>) -> io::Result<()> {
    use signal::unix::signal;
    use signal::unix::SignalKind;

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint  = signal(SignalKind::interrupt())?;

    spawn(async move {
        select! {
            _ = sigterm.recv() => {
                let _ = tx.send(SystemAction::Shutdown).await;
            }
            _ = sigint.recv() => {
                let _ = tx.send(SystemAction::Shutdown).await;
            }
        }
    });

    Ok(())
}

pub async fn run_tcp_listener(config: Arc<Config>, tx: Sender<SystemAction>) -> io::Result<()> {
    let listener = TcpListener::bind((config.tcp_ipv4.as_str(), config.tcp_port)).await?;

    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(e) => {
                eprintln!("Failed to Accept Tcp connection: {e}");
                continue;
            },
        };

        tx.send(SystemAction::SpawnTcpRouter(stream)).await
            .expect("rx shares lifetime with tx");
    }
}

pub async fn run_unix_listener(config: Arc<Config>, tx: Sender<SystemAction>) -> io::Result<()> {
    let listener = UnixListener::bind(&config.sock_path)?;

    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(e) => {
                eprintln!("Failed to Accept Unix connection: {e}");
                continue;
            },
        };

        tx.send(SystemAction::SpawnUnixRouter(stream)).await
            .expect("rx shares lifetime with tx");
    }
}

pub async fn run_unix_router(
    mut shutdown_rx: watch::Receiver<()>,
    mut stream: UnixStream, 
) -> io::Result<()> {
    let mut buf = [0u8; 512];

    let len = select! {
        biased;

        _ = shutdown_rx.changed() => {
            // abort if shutdown signal received
            return Ok(());
        }

        len = stream.read_u32() => {
            len? as usize
        }
    };

    stream.read_exact(&mut buf[..len]).await?;

    let data = &buf[..len];

    

    //TODO: check if registry contains such endpoint, if not send message

    // TODO: spawn ingest + recorder pipeline

    Ok(())
}

pub enum UnixRejectReason {
    NoSuchEndpoint,
    EndpointOccupied,
}

pub async fn run_unix_reject(
    mut stream: UnixStream, 
    endpoint:   Arc<String>,
    reason:     UnixRejectReason
) -> io::Result<()> {
    match reason {
        UnixRejectReason::NoSuchEndpoint => {
            stream.write_all(b"NoSuchEndpoint").await?;
            stream.write_all(endpoint.as_bytes()).await?;
        }

        UnixRejectReason::EndpointOccupied => {
            stream.write_all(b"Occupied").await?;
            stream.write_all(endpoint.as_bytes()).await?;
        }
    }

    Ok(())
}

// Overseer -> checks for dead tasks and tells system to restart them if necessary

pub enum IOTaskState {
    Idle,
    Processing,
}