use std::{collections::HashMap, io, sync::Arc};
use bytes::Bytes;
use tokio::{
    select, signal::{self, unix::Signal}, spawn, sync::{broadcast, mpsc, watch}, task::{self, JoinHandle}
};

use crate::{config::Config, producer, types::SystemEvent};

pub struct ProducerEntry {
    state:  producer::State,
    handle: JoinHandle<()>,
}

pub struct MachineEntry {
    // schema:      MachineEntry,
    event_tx:    broadcast::Sender<Bytes>,
    event_rx:    broadcast::Receiver<Bytes>,
    producer_id: Option<u64>,
    recorder_id: Option<u64>,
}

pub async fn run(config: Arc<Config>) -> io::Result<()> {
    let (mut sigterm, mut sigint) = create_signals()?;

    // system event channels
    let (sys_tx, mut sys_rx) = mpsc::channel::<SystemEvent>(128);

    // machine registry
    let mut machine_registry = HashMap::<&'static str, MachineEntry>::new();
    for (name, schema) in &config.machines {
        let (event_tx, event_rx) = broadcast::channel::<Bytes>(512);
        // machine_registry.insert(name, MachineEntry {
        //     schema: *schema,
        //     event_tx,
        //     event_rx,
        //     producer_id: None,
        //     recorder_id: None,
        // });
    }


    // init producer stuff
    let mut producer_registry = HashMap::<u64, ProducerEntry>::new();

    let producer_listener = producer::Listener::new(
        config.clone(),
        sys_tx.clone()
    ).await?;

    let (producer_kill_tx, producer_kill_rx) = watch::channel(());

    // start loop
    let mut id_counter = 0;
    loop {
        let event = select! {
            biased;

            _ = sigterm.recv() => { break; }
            _ = sigint.recv()  => { break; }
            opt = sys_rx.recv() => {
                opt.expect("tx/rx share lifetime")
            }
        };

        match event {
            SystemEvent::ProducerAccepted(raw_stream) => {
                use producer::{State, Stream, recv_port, execute};

                let id = id_counter;
                id_counter += 1;

                assert!(!producer_registry.contains_key(&id));
                println!("Producer Accepted and registered with Id: {id}");
                
                let stream = Stream::new(raw_stream, producer_kill_rx.clone());
                let task   = recv_port::run(stream);
                let handle = spawn(execute(sys_tx.clone(), id, task));

                producer_registry.insert(
                    id, 
                    ProducerEntry { 
                        state: State::RecvPort, 
                        handle
                    }
                );
            }

            SystemEvent::ProducerStateChanged(id, transition) => {
                use producer::{NextState, execute, recv_data};

                let entry = producer_registry.get_mut(&id).expect("Must exist");
                assert!(entry.handle.is_finished());

                let old_state = entry.state;

                match transition.next {
                    NextState::RecvData(machine) => {
                        if machine != "scales_ff01" {
                            let task   = recv_data::run(stream);
                            let handle = spawn(execute(sys_tx.clone(), id, task));
                        }

                        // entry.state = State::RecvData;
                        // let task   = recv_data::run(stream);
                        // let handle = spawn(execute(sys_tx.clone(), id, task));
                    }
                }

                println!(
                    "State Transitioned for Producer({id}) from {:?} to {:?}",
                    old_state, entry.state
                );
            }

            SystemEvent::ProducerCompleted(id, reason) => {
                println!("Producer {id} completed. Reason: {:?}", reason);
                let opt = producer_registry.remove(&id);
                assert!(opt.is_some());
            }
        }
    }

    println!("Shutting down producer listener...");
    producer_listener.stop().await;

    println!("Shutting down consumer listener...");
    // consumer_listener.stop().await;

    println!("Shutting down producers...");
    producer_kill_tx.send(()).expect("rx/tx share lifetime");

    for (id, entry) in producer_registry.drain() {
        if let Err(e) = entry.handle.await {
            // we should never cancel this, so ensure we never do
            assert!(!e.is_cancelled());
            eprintln!("Supervisor for Producer {id} panicked! {e}");
        }
    }

    println!("Shutting down recorder...");
    // TODO:

    println!("Shutting down consumers...");
    // TODO:

    Ok(())
}

fn create_signals() -> io::Result<(Signal, Signal)> {
    use signal::unix::SignalKind;
    use signal::unix::signal;

    let sigterm = signal(SignalKind::terminate())?;
    let sigint  = signal(SignalKind::interrupt())?;
    
    Ok((sigterm, sigint))
}