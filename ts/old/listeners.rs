use std::{io, sync::Arc};

use tokio::{fs, net::{TcpListener, UnixListener}, sync::mpsc};

use crate::{config::Config, root::SystemEvent};

pub async fn run_tcp(
    sys_tx: mpsc::Sender<SystemEvent>,
    config: Arc<Config>,
) -> io::Result<()> {
    let listener = TcpListener::bind((config.tcp_ipv4.as_str(), config.tcp_port)).await?;
    println!("Listening for Tcp connections on port {}", config.tcp_port);

    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(e) => {
                eprintln!("Failed to Accept Tcp connection: {e}");
                continue;
            },
        };

        sys_tx.send(SystemEvent::StartTcpGateway(stream)).await
            .expect("rx shares lifetime with tx");
    }
}

pub async fn run_unix(
    sys_tx: mpsc::Sender<SystemEvent>,
    config: Arc<Config>, 
) -> io::Result<()> {
    // try to remove the file if it exists
    _ = fs::remove_file(&config.sock_path).await;

    let listener = UnixListener::bind(&config.sock_path)?;
    println!("Listening for Unix connections on socket {}", config.sock_path);

    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(e) => {
                eprintln!("Failed to Accept Unix connection: {e}");
                continue;
            },
        };

        sys_tx.send(SystemEvent::StartUnixGateway(stream)).await
            .expect("rx shares lifetime with tx");
    }
}