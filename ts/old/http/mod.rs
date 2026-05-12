use tokio::{net::TcpStream, select, sync::{mpsc, watch}};

use crate::root::SystemEvent;

#[derive(Debug)]
pub enum Command {
    Await,
    Send   { message: String },
    Query  { machine_name: String, table: String, query: String },
    Stream { machine_name: String },
    Close,
}

#[derive(Debug)]
pub struct RedirectRequest {
    pub stream:  TcpStream,
    pub command: Command,
}

pub async fn forward_requests(
    mut rx: mpsc::Receiver<RedirectRequest>,
    tx: mpsc::Sender<SystemEvent>
) {
    loop {
        let Some(request) = rx.recv().await else {
            return;
        };

        tx.send(SystemEvent::ConsumerRedirect(request)).await
            .expect("rx must outlive tx");
    }
}