use std::{ collections::HashMap, io, sync::Arc, time::Duration };

use bytes::Bytes;
use tokio::{
    net::{
        TcpStream, 
        UnixStream
    }, select, signal::{self, unix::Signal}, spawn, sync::{
        broadcast,
        mpsc, 
        watch,
    }, task::JoinError, time::sleep 
};

use crate::{
    config::Config, gateway, http, ingest, listeners, live_stream, overseer, producer, responder, schemas::MachineDataSchema
};

#[derive(Debug)]
pub enum TerminationReason {
    ClientDisconnected,
    IoFailure(io::Error),
    JoinFailure(JoinError),
}

#[derive(Debug)]
pub enum SystemEvent {
    ProducerRedirect(producer::StateTransition),
    ProducerTerminated(TerminationReason),

    ConsumerRedirect(http::RedirectRequest),
    ConsumerTerminated(TerminationReason),

    // unix task starters
    StartUnixGateway(UnixStream),
    StartIngest(String, UnixStream),

    // Tcp task starters
    StartTcpGateway(TcpStream),
    StartLiveStream(String, TcpStream),
    StartQuery(),

    // complete task
    UnixCloseWithMessage(String, UnixStream),
    TcpCloseWithMessage(String, TcpStream),
    GetSystemStatus(TcpStream),
}

pub struct MachineEntry {
    schema:   MachineDataSchema,
    event_tx: broadcast::Sender<Bytes>,
    event_rx: broadcast::Receiver<Bytes>,
}

// unix connections
pub struct ProducerConnection {
    address: String,
    state:   u32,
}

// http connections
pub struct ConsumerConnection {
    address: String,
    state:   u32,
}

pub async fn run(config: Arc<Config>)  {
    let (mut sigterm, mut sigint) = create_signals();

    // system action channels
    let (sys_tx, mut sys_rx) = mpsc::channel::<SystemEvent>(128);

    // in router (unix)
    // let (http_router_tx, mut http_router_rx) = mpsc::channel::<http::RedirectRequest>(128);
    // let http_router = spawn(http::forward_requests(http_router_rx, sys_tx.clone()));

    // let in_connections = Vec::new();

    // out router (http)
    let (http_router_tx, mut http_router_rx) = mpsc::channel::<http::RedirectRequest>(128);
    let http_router = spawn(http::forward_requests(http_router_rx, sys_tx.clone()));

    // listeners
    let unix_listener = spawn(listeners::run_unix(sys_tx.clone(), config.clone()));
    let tcp_listener  = spawn(listeners::run_tcp(sys_tx.clone(),  config.clone()));

    // routers
    let (router_ov_tx, router_ov_rx) = mpsc::channel(32);
    let router_ov = spawn(overseer::run("router", router_ov_rx));

    // responders
    let (responder_ov_tx, responder_ov_rx) = mpsc::channel(32);
    let responder_ov = spawn(overseer::run("responder", responder_ov_rx));

    // ingest
    let (ingest_kill_tx, ingest_kill_rx) = watch::channel(());
    let (ingest_ov_tx, ingest_ov_rx) = mpsc::channel(32);
    let mut ingest_ov = spawn(overseer::run("ingest", ingest_ov_rx));

    // live
    let (live_ov_tx, live_ov_rx) = mpsc::channel(32);
    let mut live_ov = spawn(overseer::run("live", live_ov_rx));

    // query
    // TODO: implement

    let mut in_connections = HashMap::new();

    let mut entries = HashMap::new();
    for (name, schema) in &config.machines {
        let (event_tx, event_rx) = broadcast::channel::<Bytes>(512);
        entries.insert(name, MachineEntry {
            schema: *schema,
            event_tx,
            event_rx,
        });
    }

    loop {
        let action = select! {
            biased;

            _ = sigterm.recv() => { break; }
            _ = sigint.recv()  => { break; }
            action = sys_rx.recv() => {
                action.expect("tx may not be dropped")
            }
        };

        match action {
            SystemEvent::ProducerRedirect(request) => {
                use producer::State::*;

                match request.next {
                    Await => {
                        
                    }

                    Ingest(_) => {

                    }

                    Exit => {
                        // drop connection
                    }
                }

                // let task = producer::ingest::run(exit_rx, out_tx, stream, validate);
            }

            SystemEvent::StartUnixGateway(stream) => {
                let task = gateway::unix::run(sys_tx.clone(), config.clone(), stream);
                router_ov_tx.try_send(Box::pin(task))
                    .expect("Channel may not be unavailable");
            }

            SystemEvent::StartTcpGateway(stream) => {
                let task = gateway::tcp::run(sys_tx.clone(), stream);
                router_ov_tx.try_send(Box::pin(task))
                    .expect("Channel may not be unavailable");
            }

            SystemEvent::StartIngest(machine_name, stream) => {
                let Some(entry) = entries.get_mut(&machine_name) else {
                    eprintln!("machine_name {machine_name} not found!");
                    //TODO : respond with internal server error
                    continue;
                };

                let task = ingest::run(
                    ingest_kill_rx.clone(), 
                    entry.event_tx.clone(), 
                    stream, 
                    entry.schema.validate,
                );

                ingest_ov_tx.try_send(Box::pin(task))
                    .expect("Channel may not be unavailable");
            }

            SystemEvent::StartLiveStream(machine_name, stream) => {
                let Some(entry) = entries.get_mut(&machine_name) else {
                    eprintln!("machine_name {machine_name} not found!");
                    //TODO : respond with internal server error
                    continue;
                };

                let event_rx = entry.event_tx.subscribe();
                let task = live_stream::run(event_rx, stream);

                live_ov_tx.try_send(Box::pin(task))
                    .expect("Channel may not be unavailable");
            }
                
            SystemEvent::StartQuery() => {
                todo!("Lmao");
            }

            SystemEvent::TcpCloseWithMessage(message, stream) => {
                let task = responder::run(stream, message);
                responder_ov_tx.try_send(Box::pin(task))
                    .expect("Channel may not be unavailable");
            }

            SystemEvent::UnixCloseWithMessage(message, stream) => {
                let task = responder::run(stream, message);
                responder_ov_tx.try_send(Box::pin(task))
                    .expect("Channel may not be unavailable");
            }
        }
    }

    println!("System shutdown started");

    println!("Stopping unix listener...");
    unix_listener.abort();

    println!("Stopping tcp listener...");
    tcp_listener.abort();

    println!("Stopping routers...");
    router_ov.abort();

    println!("Stopping ingestors...");
    ingest_kill_tx.send(()).expect("rx shares lifetime with tx");
    select! {
        biased;

        join_result = &mut ingest_ov => {
            join_result.expect("Cannot fail");
        }

        _ = sleep(Duration::from_secs(2)) => {
            // timeout exceeded
            ingest_ov.abort();
        }
    }

    println!("Stopping live...");
    select! {
        biased;

        join_result = &mut live_ov => {
            join_result.expect("Cannot fail");
        }

        _ = sleep(Duration::from_secs(2)) => {
            // timeout exceeded
            live_ov.abort();
        }
    }

    //TODO: give 2 second grace period ?
    println!("Stopping responders...");
    responder_ov.abort();

    // stop query ov
}

fn create_signals() -> (Signal, Signal) {
    use signal::unix::SignalKind;
    use signal::unix::signal;

    let sigterm = signal(SignalKind::terminate())
        .expect("Signals must be available");

    let sigint  = signal(SignalKind::interrupt())
        .expect("Signals must be available");

    (sigterm, sigint)
}