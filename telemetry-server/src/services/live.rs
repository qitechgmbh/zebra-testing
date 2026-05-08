use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use crossbeam::channel::{Sender, TrySendError, bounded};

use crate::{Payload, PayloadReceiver, PayloadSender, config::Config};











pub fn run(config: Arc<Config>, rx: Arc<PayloadReceiver>) -> anyhow::Result<()> {
    let (clients_tx, clients_rx) = bounded::<PayloadSender>(config.live_channel_capacity);

    let _ = thread::spawn(move || run_accept(config, clients_tx));

    let mut clients: Vec<PayloadSender> = Vec::new();

    for msg in rx.iter() {
        let mut dead = Vec::new();

        // append new clients into list
        if let Ok(client_rx) = clients_rx.try_recv() {
            clients.push(client_rx);
        }

        // send message to each client and 
        // remove clients where sending fails
        for (i, tx) in clients.iter().enumerate() {
            match tx.try_send(msg.clone()) {
                Ok(_) => {}
                Err(TrySendError::Disconnected(_)) => {
                    dead.push(i);
                }
                Err(TrySendError::Full(_)) => {
                    eprintln!("Client failed to retrieve data in time → disconnecting");
                    dead.push(i);
                }
            }
        }

        // remove dead clients (from back to front)
        for i in dead.into_iter().rev() {
            clients.remove(i);
        }
    }

    Ok(())
}

fn run_accept(config: Arc<Config>, clients_tx: Sender<PayloadSender>) -> anyhow::Result<()> {
    let address = format!("0.0.0.0:{}", config.live_port);
    let listener = TcpListener::bind(address)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let (tx, rx_client) = bounded::<Payload>(config.live_channel_capacity);

                // give server the Sender for the client
                clients_tx.send(tx)?;

                // start client handler
                thread::spawn(move || handle_client(stream, rx_client));
            }
            Err(e) => eprintln!("accept error: {e}"),
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, rx: PayloadReceiver) -> anyhow::Result<()> {
    for msg in rx.iter() {
        let len = msg.len() as u16;
        stream.write_all(&len.to_le_bytes())?;
        stream.write_all(&msg)?
    }

    Ok(())
}
