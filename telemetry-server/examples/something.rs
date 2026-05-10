use std::{collections::HashMap, io};

use actix::{Actor, Context, Recipient};
use tokio::{net::{TcpListener, UnixListener, UnixStream}, spawn, sync::mpsc::{Receiver, Sender}};




// live is event based
// live listener -> 

// live event: new message, new client, terminate

// one central loop that accepts tcp connections and routes them correctly
// that central loop can never die, doesn't need protecetion. 
// On client we just spawn a new live handler with a message input for new messages
// we know when the message queue is completed, so can assume when client is finished


// system consists of 

// event bus: 
// address:    scales_ff01, scales_ff02
// recipients: Overseer, Ingest, Record, Live, Query



// Actors: 

pub enum Event {
    Shutdown,
    Terminated(anyhow::Error),
}

pub enum IoTaskState {
    Idle,
    Pending,
}

pub enum RecipientType {
    System,
    Ingest,
    Recorder,
    Live,
    Query,
}

pub struct Recipients {
    system: bool,
    ingest: bool,
    live:   bool,
    query:  bool,
}

pub struct EventMessage {
    bus_id:     u8,
    recipients: Recipients,
    event:      Event,
}

struct EventRouter {
    rx: Receiver<EventMessage>,

    system_rx: Receiver<Event>,

    connections: HashMap<u8, HashMap<RecipientType, Vec<Receiver<Event>>>>
}

struct EventBusConnection {
    bus_id: u8,
    tx: Sender<EventMessage>,
    rx: Receiver<Event>,
}

impl EventBusConnection {
    pub async fn recv(&mut self) -> Event {
        self.rx.recv().await.expect("EventRouter must outlive connection")
    }

    pub async fn send(&mut self, event: Event, recipients: Recipients) {
        let message = EventMessage { bus_id: self.bus_id, recipients, event };
        self.tx.send(message).await.expect("EventRouter must outlive connection")
    }
}

struct EventBusSender {
    bus_id: u8,
    tx: Sender<EventMessage>,
}

impl EventBusSender {
    pub async fn send(&self, event: Event, recipients: Recipients) {
        let message = EventMessage { bus_id: self.bus_id, recipients, event };
        self.tx.send(message).await.expect("EventRouter must outlive connection")
    }
}


pub async fn run_io_task<F, Fut, Ctx>(
    sender: EventBusSender,
    mut ctx: Ctx,
    mut cycle: F,
)
where
    F: FnMut(&mut Ctx) -> Fut,
    Fut: Future<Output = anyhow::Result<()>>,
    Ctx: Send,
{
    let handle = spawn(async move {
        loop {
            if let Err(e) = cycle(&mut ctx).await {
                let recipients = Recipients {
                    system: true,
                    ingest: false,
                    live:   false,
                    query:  false,
                };

                sender.send(Event::Terminated(e), recipients);
                break;
            }
        }
    });

    if let Err(e) = handle.await {
        
    }
}

// TelemetryEventValidator

// TaskRunner: runs the loop, notifies system of crash/error

pub async fn run_task<F: Future>(sender: EventBusSender, cycle: , ctx) {

}

// io task
pub async fn ingest_task(sender: EventBusSender, stream: UnixStream) {
    loop {
        
    }
}

pub async fn ingest_task_cycle(stream: &UnixStream) {
    
}

struct QueryTask {

    //recipient: Recipient<Ping>,
}

pub trait EventTask {

}

// register service, provide a function  that launches the service

pub async fn run_connector() {

}

pub struct System {
    registry: HashMap<String, String>,

    unix_listener: UnixListener,

}

impl System {
    pub async fn run(
        unix_listener: UnixListener, 
        tcp_listener:  TcpListener
    ) -> io::Result<()> {
        let unix_handle = spawn(run_unix_listener(unix_listener));
        let tcp_handle  = spawn(run_tcp_listener(tcp_listener));

        // recorders

        // start receiving events 

        unix_handle.abort();
        tcp_handle.abort();

        Ok(())
    }
}

pub async fn run_unix_listener(listener: UnixListener) -> io::Result<()> {
    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(e) => {
                eprintln!("Failed to Accept Unix connection: {e}");
                continue;
            },
        };

        // TODO: send stream to System to start ingest
    }
}

pub async fn run_tcp_listener(listener: TcpListener) -> io::Result<()> {
    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(e) => {
                eprintln!("Failed to Accept Unix connection: {e}");
                continue;
            },
        };

        // TODO: send stream to System to start ingest
    }
}

pub async fn supervise(/*  */) {

}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut registry = Vec::new();
    registry.push(("scales_ff01", "scales_s0"));

    for item in registry {

    }

    println!("Hello World!");
    Ok(())
}

// initialize machine -> create ingest for each machine
// 

// each machine gets schema + pipeline

// schema consists of event payload + database

// ENV VAR: machine name + schema